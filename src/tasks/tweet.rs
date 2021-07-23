use crate::{
  client::redis::Redis,
  common::redis::{gbf_persistence_raid_tweet_key, gbf_raid_boss_raw_key},
  error,
  models::{Language, TranslatorResult, Tweet},
  parsers::status::StatusParser,
  proto::{raid_boss_raw::RaidBossRaw, raid_tweet::RaidTweet},
  resources::{
    redis::{BOSS_EXPIRE_IN_30_DAYS_TTL, TWEET_PERSISTENCE_ONLY_2_HOURS_TTL},
    GRANBLUE_FANTASY_SOURCE,
  },
  tasks::translator,
  Result,
};

use futures::TryFutureExt;
use log::{debug, error};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::{mpsc, oneshot, RwLock};

enum TweetActorMessage {
  ///
  /// If the tweet is came from Granblue Fantasy source using
  /// StatusParser to parse the tweet to raid_tweet and raid_boss_raw.
  /// If it is from others source throw an CannotParseTweet error.
  ///
  /// # Arguments
  /// * `tweet` - the original tweet which came from twitter streaming api.
  ///
  ParseTweet {
    tweet: Tweet,
    respond_to: oneshot::Sender<Result<(RaidBossRaw, RaidTweet)>>,
  },
  ///
  /// Check whether actor translator map has the raid_boss_row.boss_name().
  /// If translator map has the name, just return the translated name.
  /// If translator map has the name but it is an empty string, means that translation task is pending.
  /// If not create a task to do the translation.
  ///
  /// # Arguments
  /// * `raid_boss_raw` - the raid_boss which we want to translate.
  ///
  TranslateBossName {
    raid_boss_raw: RaidBossRaw,
    respond_to: oneshot::Sender<TranslatorResult>,
  },
  ///
  /// Translate the english boss name raid_tweet to japanese name.
  ///
  /// # Arguments
  /// * `raid_boss_raw` - use to show the error message of boss name.
  /// * `raid_tweet` - raid tweet which should be translated.
  /// * `translator_result` - result from TranslateBossName
  TranslateTweet {
    raid_boss_raw: RaidBossRaw,
    raid_tweet: RaidTweet,
    translator_result: TranslatorResult,
    respond_to: oneshot::Sender<Result<RaidTweet>>,
  },
  ///
  /// Persist the raid tweet into redis.
  ///
  /// # Arguments
  /// * `raid_tweet` - raid_tweet which should be persisted.
  PersistRaidTweet {
    raid_tweet: RaidTweet,
    respond_to: oneshot::Sender<Result<RaidTweet>>,
  },
}

struct TweetActor {
  receiver: mpsc::Receiver<TweetActorMessage>,
  redis: Arc<Redis>,
  map: Arc<RwLock<HashMap<String, String>>>,
}

impl TweetActor {
  pub fn new(receiver: mpsc::Receiver<TweetActorMessage>, redis: Arc<Redis>, map: HashMap<String, String>) -> Self {
    TweetActor {
      receiver,
      redis,
      map: Arc::new(RwLock::new(map)),
    }
  }

  async fn run(&mut self) {
    while let Some(msg) = self.receiver.recv().await {
      if let Err(error) = self.handle_message(msg).await {
        error!("Error encounter during actor, error: {:?}", error);
      }
    }
  }

  async fn handle_message(&mut self, msg: TweetActorMessage) -> Result<()> {
    match msg {
      TweetActorMessage::ParseTweet { tweet, respond_to } => match tweet.source.as_str() {
        // Only process tweet from granblue fantasy source
        GRANBLUE_FANTASY_SOURCE => {
          if let Some((raid_bow_raw, raid_tweet)) = StatusParser::parse(tweet) {
            let redis_key = gbf_raid_boss_raw_key(&raid_bow_raw);
            // Each boss will only have 30 days ttl
            self
              .redis
              .set_protobuf(&redis_key, raid_bow_raw.clone(), BOSS_EXPIRE_IN_30_DAYS_TTL)
              .await?;
            let _ = respond_to.send(Ok((raid_bow_raw, raid_tweet)));
          }

          Ok(())
        }
        _ => {
          debug!("Twitter filter stream find the source which is not from granblue fantasy");
          let _ = respond_to.send(Err(error::Error::CannotParseTweet { tweet }));

          Ok(())
        }
      },
      TweetActorMessage::TranslateBossName {
        raid_boss_raw,
        respond_to,
      } => {
        let translate_map = self.map.read().await;
        // Return directly if boss_name is already translated.
        match translate_map.get(raid_boss_raw.get_boss_name()) {
          Some(translated) => {
            // If value in map is an empty string, it indicate that the translation process is processing.
            match translated.is_empty() {
              true => {
                debug!("Translating task of {} is pending...", raid_boss_raw.get_boss_name());
                let _ = respond_to.send(TranslatorResult::Pending);
              }
              false => {
                let _ = respond_to.send(TranslatorResult::Success {
                  result: translated.to_string(),
                });
              }
            };

            Ok(())
          }
          None => {
            // Drop map RwLock before translating
            drop(translate_map);
            let mut writable_map = self.map.write().await;
            // Write an empty string to `map` means that translation is pending.
            writable_map.insert(raid_boss_raw.get_boss_name().into(), "".into());
            drop(writable_map);
            // Response to handler before processing translation tasks.
            let _ = respond_to.send(TranslatorResult::Pending);
            debug!("Find new boss {}. Translating...", raid_boss_raw.get_boss_name());

            // Prepare for translation task.
            let map = self.map.clone();
            let redis = self.redis.clone();

            // Do translation parallel
            tokio::spawn(async move {
              translator::translator_tasks(raid_boss_raw, redis, map).await?;

              Ok::<(), error::Error>(())
            });

            Ok(())
          }
        }
      }
      TweetActorMessage::TranslateTweet {
        raid_boss_raw,
        mut raid_tweet,
        translator_result,
        respond_to,
      } => {
        let language = Language::from_str(raid_boss_raw.get_language()).unwrap();
        let translated_tweet = match language {
          // Only English boss name should be converted into Japanese
          Language::English => match translator_result {
            TranslatorResult::Pending => Err(error::Error::CannotTranslate {
              name: raid_boss_raw.get_boss_name().into(),
            }),
            TranslatorResult::Success {
              result: translated_name,
            } => {
              if language == Language::English {
                raid_tweet.set_boss_name(translated_name);
              }

              Ok(raid_tweet)
            }
          },
          Language::Japanese => Ok(raid_tweet),
        };

        let _ = respond_to.send(translated_tweet);

        Ok(())
      }
      TweetActorMessage::PersistRaidTweet { raid_tweet, respond_to } => {
        let _ = respond_to.send(Ok(raid_tweet.clone()));

        let redis = self.redis.clone();

        // Persist raid_tweet parallel
        tokio::spawn(async move {
          redis
            .set_protobuf(
              gbf_persistence_raid_tweet_key(raid_tweet.get_boss_name(), raid_tweet.tweet_id, raid_tweet.created),
              raid_tweet,
              TWEET_PERSISTENCE_ONLY_2_HOURS_TTL,
            )
            .await?;

          Ok::<(), error::Error>(())
        });

        Ok(())
      }
    }
  }
}

pub struct TweetActorHandle {
  sender: mpsc::Sender<TweetActorMessage>,
}

impl TweetActorHandle {
  pub fn new(redis: Arc<Redis>, map: HashMap<String, String>) -> Self {
    let (sender, receiver) = mpsc::channel(1024);
    let mut actor = TweetActor::new(receiver, redis, map);
    let _ = tokio::spawn(async move { actor.run().await }).map_err(|e| {
      if e.is_panic() {
        error!("Actor task might get panic!, error: {}", e);
      } else if e.is_cancelled() {
        error!("Actor task might get cancelled, error: {}", e);
      }
    });

    Self { sender }
  }

  pub async fn parse_tweet(&self, tweet: Tweet) -> Result<(RaidBossRaw, RaidTweet)> {
    let (send, recv) = oneshot::channel();
    let msg = TweetActorMessage::ParseTweet {
      tweet: tweet.clone(),
      respond_to: send,
    };
    let _ = self.sender.send(msg).await;
    let result = recv.await;
    match result {
      Ok(result) => result,
      Err(e) => Err(error::Error::ActorTaskBeenKilled { error: e }),
    }
  }

  pub async fn translate_boss_name(&self, raid_boss_raw: RaidBossRaw) -> Result<TranslatorResult> {
    let (send, recv) = oneshot::channel();
    let msg = TweetActorMessage::TranslateBossName {
      raid_boss_raw,
      respond_to: send,
    };
    let _ = self.sender.send(msg).await;
    let result = recv.await;
    match result {
      Ok(result) => Ok(result),
      Err(e) => Err(error::Error::ActorTaskBeenKilled { error: e }),
    }
  }

  pub async fn translate_tweet(
    &self,
    raid_boss_raw: RaidBossRaw,
    raid_tweet: RaidTweet,
    translator_result: TranslatorResult,
  ) -> Result<RaidTweet> {
    let (send, recv) = oneshot::channel();
    let msg = TweetActorMessage::TranslateTweet {
      raid_boss_raw,
      raid_tweet,
      translator_result,
      respond_to: send,
    };
    let _ = self.sender.send(msg).await;
    let result = recv.await;
    match result {
      Ok(result) => result,
      Err(e) => Err(error::Error::ActorTaskBeenKilled { error: e }),
    }
  }

  pub async fn persist_raid_tweet(&self, raid_tweet: RaidTweet) -> Result<RaidTweet> {
    let (send, recv) = oneshot::channel();
    let msg = TweetActorMessage::PersistRaidTweet {
      raid_tweet,
      respond_to: send,
    };
    let _ = self.sender.send(msg).await;
    let result = recv.await;
    match result {
      Ok(result) => result,
      Err(e) => Err(error::Error::ActorTaskBeenKilled { error: e }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    models::{Entity, Language, Media, User},
    resources::redis::BOSS_EXPIRE_IN_30_DAYS_TTL,
    Result,
  };
  use std::env;

  lazy_static::lazy_static! {
    static ref JP_TWEET: Tweet = Tweet {
      id: 1390247452125458434,
      text: "麻痹延长 7D705AE2 :参戦ID\n参加者募集！\nLv150 プロトバハムート\nhttps://t.co/MYfvDDTSrh".into(),
      source: r#"<a href="http://granbluefantasy.jp/" rel="nofollow">グランブルー ファンタジー</a>"#.into(),
      entities: Entity {
        media: Some(vec![Media {
          media_url_https: "https://pbs.twimg.com/media/CdL4WyxUYAIXPb8.jpg".into(),
        }]),
      },
      timestamp_ms: "1620698515453".to_string(),
      user: User {
        screen_name: "".to_string(),
        profile_image_url_https: "".to_string(),
      },
    };
    static ref EN_TWEET: Tweet = Tweet {
      id: 1390247452125458434,
      text:
        "I love granblue fantasy 7D705AE2 :Battle ID\nI need backup!\nLvl 150 Proto Bahamut\nhttps://t.co/MYfvDDTSrh"
          .into(),
      source: r#"<a href="http://granbluefantasy.jp/" rel="nofollow">グランブルー ファンタジー</a>"#.into(),
      entities: Entity {
        media: Some(vec![Media {
          media_url_https: "https://pbs.twimg.com/media/CdL4WyxUYAIXPb8.jpg".into(),
        }]),
      },
      timestamp_ms: "1620698515453".to_string(),
      user: User {
        screen_name: "".to_string(),
        profile_image_url_https: "".to_string(),
      },
    };
    static ref REDIS_URL: String  = env::var("REDIS_URL").unwrap();
    static ref JP_RAID_BOSS_RAW: RaidBossRaw = RaidBossRaw::with_args("Lv150 プロトバハムート", 150, "https://pbs.twimg.com/media/CdL4WyxUYAIXPb8.jpg", Language::Japanese);
    static ref EN_RAID_BOSS_RAW: RaidBossRaw = RaidBossRaw::with_args("Lvl 150 Proto Bahamut", 150, "https://pbs.twimg.com/media/CfqZ-YtVAAAt5qd.jpg", Language::English);
  }

  #[tokio::test]
  async fn test_jp_tweet_actor_translation() -> Result<()> {
    let redis = Redis::new(REDIS_URL.clone())?;
    let redis = Arc::new(redis);
    let map: HashMap<String, String> = HashMap::new();
    redis
      .set_protobuf(
        gbf_raid_boss_raw_key(&EN_RAID_BOSS_RAW),
        EN_RAID_BOSS_RAW.clone(),
        BOSS_EXPIRE_IN_30_DAYS_TTL,
      )
      .await?;
    redis
      .set_protobuf(
        gbf_raid_boss_raw_key(&JP_RAID_BOSS_RAW),
        JP_RAID_BOSS_RAW.clone(),
        BOSS_EXPIRE_IN_30_DAYS_TTL,
      )
      .await?;
    let actor = TweetActorHandle::new(redis, map);
    let (raid_boss_raw, _raid_tweet) = actor.parse_tweet(JP_TWEET.clone()).await.unwrap();
    assert_eq!(
      actor.translate_boss_name(raid_boss_raw.clone()).await.unwrap(),
      TranslatorResult::Pending
    );
    let mut max_retry = 5;
    let mut translated_name = String::from("");
    while max_retry != 0 {
      // Sleep 10 seconds to wait for translation task
      tokio::time::sleep(std::time::Duration::from_secs(10)).await;
      match actor.translate_boss_name(raid_boss_raw.clone()).await.unwrap() {
        TranslatorResult::Pending => {
          max_retry -= 1;
        }
        TranslatorResult::Success { result: translated } => {
          translated_name = translated.to_owned();
          break;
        }
      };
    }
    assert_eq!("Lvl 150 Proto Bahamut", translated_name.to_string());
    Ok(())
  }

  #[tokio::test]
  async fn test_jp_tweet_actor_already_translated() -> Result<()> {
    let redis = Redis::new(REDIS_URL.clone())?;
    let redis = Arc::new(redis);
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("Lv150 プロトバハムート".into(), "Lvl 150 Proto Bahamut".into());
    map.insert("Lvl 150 Proto Bahamut".into(), "Lv150 プロトバハムート".into());
    let actor = TweetActorHandle::new(redis, map);
    let (raid_boss_raw, raid_tweet) = actor.parse_tweet(JP_TWEET.clone()).await.unwrap();
    assert_eq!(raid_boss_raw.boss_name, "Lv150 プロトバハムート");
    assert_eq!(raid_boss_raw.level, 150);
    assert_eq!(raid_boss_raw.image, "https://pbs.twimg.com/media/CdL4WyxUYAIXPb8.jpg");
    assert_eq!(raid_tweet.boss_name, "Lv150 プロトバハムート");
    assert_eq!(raid_tweet.tweet_id, 1390247452125458434);
    let translated_name = actor.translate_boss_name(raid_boss_raw).await.unwrap();
    assert_eq!("Lvl 150 Proto Bahamut", translated_name.to_string());

    Ok(())
  }

  #[tokio::test]
  async fn test_en_tweet_already_translated() -> Result<()> {
    let redis = Redis::new(REDIS_URL.clone())?;
    let redis = Arc::new(redis);
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("Lv150 プロトバハムート".into(), "Lvl 150 Proto Bahamut".into());
    map.insert("Lvl 150 Proto Bahamut".into(), "Lv150 プロトバハムート".into());
    let actor = TweetActorHandle::new(redis, map);
    let (raid_boss_raw, raid_tweet) = actor.parse_tweet(EN_TWEET.clone()).await.unwrap();
    assert_eq!(raid_boss_raw.boss_name, "Lvl 150 Proto Bahamut");
    assert_eq!(raid_boss_raw.level, 150);
    assert_eq!(raid_boss_raw.image, "https://pbs.twimg.com/media/CdL4WyxUYAIXPb8.jpg");
    assert_eq!(raid_tweet.boss_name, "Lvl 150 Proto Bahamut");
    assert_eq!(raid_tweet.tweet_id, 1390247452125458434);
    let translated_name = actor.translate_boss_name(raid_boss_raw).await.unwrap();
    assert_eq!("Lv150 プロトバハムート", translated_name.to_string());

    Ok(())
  }
}
