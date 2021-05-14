use crate::proto::raid_tweet::RaidTweet;
use crate::{
  client::{filter_stream::StreamingSource, redis::Redis},
  common::redis::{gbf_persistence_raid_tweet_key, gbf_raid_boss_raw_key},
  error,
  models::Tweet,
  parsers::status::StatusParser,
  proto::raid_boss_raw::RaidBossRaw,
  resources::{BOSS_EXPIRE_IN_30_DAYS_TTL, GRANBLUE_FANTASY_SOURCE, TWEET_PERSISTENCE_ONLY_2_HOURS_TTL},
  tasks::translator,
  Result,
};
use log::{debug, error, info};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, oneshot, RwLock};

enum TweetActorMessage {
  ParseTweet {
    tweet: Tweet,
    respond_to: oneshot::Sender<Option<(RaidBossRaw, RaidTweet)>>,
  },
  TranslateBossName {
    raid_boss: RaidBossRaw,
    respond_to: oneshot::Sender<Option<String>>,
  },
  PersistRaidTweet {
    raid_tweet: RaidTweet,
    respond_to: oneshot::Sender<()>,
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
            let _ = respond_to.send(Some((raid_bow_raw, raid_tweet)));
          }

          Ok(())
        }
        _ => {
          debug!("Twitter filter stream find the source which is not from granblue fantasy");
          let _ = respond_to.send(None);

          Ok(())
        }
      },
      TweetActorMessage::TranslateBossName { raid_boss, respond_to } => {
        let translate_map = self.map.read().await;
        // Return directly if boss_name is already translated.
        match translate_map.get(raid_boss.get_boss_name()) {
          Some(translated) => {
            let _ = respond_to.send(Some(translated.into()));

            Ok(())
          }
          None => {
            // Drop map RwLock before translating
            drop(translate_map);
            let mut writable_map = self.map.write().await;
            // Write an empty string to `map` means that translation is pending.
            writable_map.insert(raid_boss.get_boss_name().into(), "".into());
            drop(writable_map);
            // Response to handler before processing translation tasks.
            let _ = respond_to.send(None);
            info!("Find new boss {}. Translating...", raid_boss.get_boss_name());
            let map = self.map.clone();
            let redis = self.redis.clone();

            tokio::spawn(async move {
              translator::translator_tasks(raid_boss, redis, map).await?;

              Ok::<(), error::Error>(())
            });

            Ok(())
          }
        }
      }
      TweetActorMessage::PersistRaidTweet { raid_tweet, respond_to } => {
        let _ = respond_to.send(());

        let redis = self.redis.clone();
        tokio::spawn(async move {
          redis
            .set_protobuf(
              gbf_persistence_raid_tweet_key(raid_tweet.get_boss_name(), raid_tweet.tweet_id),
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

async fn run_my_actor(mut actor: TweetActor) {
  while let Some(msg) = actor.receiver.recv().await {
    if let Err(error) = actor.handle_message(msg).await {
      error!("Error encounter during actor, error: {:?}", error);
    }
  }
}

#[derive(Clone)]
pub struct TweetActorHandle {
  sender: mpsc::Sender<TweetActorMessage>,
}

impl TweetActorHandle {
  pub fn new(redis: Arc<Redis>, map: HashMap<String, String>) -> Self {
    let (sender, receiver) = mpsc::channel(1024);
    let actor = TweetActor::new(receiver, redis, map);
    tokio::spawn(run_my_actor(actor));

    Self { sender }
  }

  pub async fn parse_tweet(&self, tweet: Tweet) -> Result<Option<(RaidBossRaw, RaidTweet)>> {
    let (send, recv) = oneshot::channel();
    let msg = TweetActorMessage::ParseTweet {
      tweet: tweet.clone(),
      respond_to: send,
    };
    let _ = self.sender.send(msg).await;
    recv.await.map_err(|_| error::Error::CannotParseTweet { tweet })
  }

  pub async fn translate_boss_name(&self, raid_boss_raw: RaidBossRaw) -> Result<Option<String>> {
    let (send, recv) = oneshot::channel();
    let msg = TweetActorMessage::TranslateBossName {
      raid_boss: raid_boss_raw.clone(),
      respond_to: send,
    };
    let _ = self.sender.send(msg).await;
    recv.await.map_err(|_| error::Error::CannotTranslateError {
      name: raid_boss_raw.boss_name,
    })
  }

  pub async fn persist_raid_tweet(&self, raid_tweet: RaidTweet) {
    let (send, recv) = oneshot::channel();
    let msg = TweetActorMessage::PersistRaidTweet {
      raid_tweet,
      respond_to: send,
    };
    let _ = self.sender.send(msg).await;
    recv.await.expect("Actor task has been killed");
  }
}
