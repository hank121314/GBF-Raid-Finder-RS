use crate::{
  client::redis::Redis,
  common::redis::{gbf_get_possible_boss_name, gbf_raid_boss_key, gbf_translator_key},
  error,
  image::Comparison,
  models::{Language, Tweet},
  parsers::status::StatusParser,
  proto::raid_boss::RaidBoss,
  resources::{BOSS_EXPIRE_IN_30_DAYS_TTL, GRANBLUE_FANTASY_SOURCE},
  Result,
};
use log::{error, info};
use std::{borrow::Borrow, collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::{mpsc, oneshot, RwLock};

enum ActorMessage {
  ParseTweet {
    tweet: Tweet,
    respond_to: oneshot::Sender<Option<RaidBoss>>,
  },
  TranslateBossName {
    raid_boss: RaidBoss,
    respond_to: oneshot::Sender<Option<String>>,
  },
}

struct Actor {
  receiver: mpsc::Receiver<ActorMessage>,
  redis: Arc<Redis>,
  map: Arc<RwLock<HashMap<String, String>>>,
}

impl Actor {
  pub fn new(receiver: mpsc::Receiver<ActorMessage>, redis: Redis, map: HashMap<String, String>) -> Self {
    Actor {
      receiver,
      redis: Arc::new(redis),
      map: Arc::new(RwLock::new(map)),
    }
  }

  async fn handle_message(&mut self, msg: ActorMessage) -> Result<()> {
    match msg {
      ActorMessage::ParseTweet { tweet, respond_to } => {
        if tweet.source == GRANBLUE_FANTASY_SOURCE {
          if let Some(raid) = StatusParser::parse(tweet) {
            let language = Language::from_str(raid.get_language()).unwrap();
            let redis_key = gbf_raid_boss_key(raid.borrow(), language);
            self
              .redis
              .set_protobuf(&redis_key, raid.clone(), BOSS_EXPIRE_IN_30_DAYS_TTL)
              .await?;
            let _ = respond_to.send(Some(raid));
            return Ok(());
          }
        } else {
          error!("Twitter filter stream find the source which is not from granblue fantasy");
        }

        let _ = respond_to.send(None);

        Ok(())
      }
      ActorMessage::TranslateBossName { raid_boss, respond_to } => {
        let language = match raid_boss.get_language() {
          "English" => Language::Japanese,
          "Japanese" => Language::English,
          _ => Language::Japanese,
        };
        if language == Language::English {
          let _ = respond_to.send(Some(raid_boss.get_boss_name().into()));

          return Ok(());
        };
        let translate_map = self.map.read().await;
        return match translate_map.get(raid_boss.get_boss_name()) {
          None => {
            let _ = respond_to.send(None);
            drop(translate_map);
            info!("Find new boss {}. Translating...", raid_boss.get_boss_name());
            let map = self.map.clone();
            let redis = self.redis.clone();
            tokio::spawn(async move {
              let possible_name = gbf_get_possible_boss_name(raid_boss.clone(), language);
              let possible_boss_keys = redis.keys(possible_name).await?;
              let possible_bosses = redis.mget_protobuf(possible_boss_keys).await?;
              println!("{:?}", possible_bosses);
              let comparison = Comparison::new(raid_boss.clone(), possible_bosses);
              if let Some(matched) = comparison.compare().await? {
                let mut writable_map = map.write().await;
                writable_map.insert(raid_boss.get_boss_name().into(), matched.get_boss_name().into());
                let map_2_redis: HashMap<String, String> = writable_map.iter().map(|k| (k.0.into(), k.1.into())).collect::<HashMap<_, _>>();
                drop(writable_map);
                info!("Translate {} name to {} complete! Writing to redis...", raid_boss.get_boss_name(), matched.get_boss_name());
                redis
                  .set_hash_map(gbf_translator_key(), map_2_redis, 0)
                  .await?;
              }

              Ok::<(), error::Error>(())
            });

            Ok(())
          }
          Some(translated) => {
            let _ = respond_to.send(Some(translated.into()));

            Ok(())
          }
        };
      }
    }
  }
}

async fn run_my_actor(mut actor: Actor) {
  while let Some(msg) = actor.receiver.recv().await {
    if let Err(error) = actor.handle_message(msg).await {
      error!("Error encounter during actor, error: {:?}", error);
    }
  }
}

#[derive(Clone)]
pub struct ActorHandle {
  sender: mpsc::Sender<ActorMessage>,
}

impl ActorHandle {
  pub fn new(redis: Redis, map: HashMap<String, String>) -> Self {
    let (sender, receiver) = mpsc::channel(1024);
    let actor = Actor::new(receiver, redis, map);
    tokio::spawn(run_my_actor(actor));

    Self { sender }
  }

  pub async fn parse_tweet(&self, tweet: Tweet) -> Result<RaidBoss> {
    let (send, recv) = oneshot::channel();
    let msg = ActorMessage::ParseTweet {
      tweet: tweet.clone(),
      respond_to: send,
    };
    let _ = self.sender.send(msg).await;
    recv
      .await
      .map_err(|_| error::Error::SenderSendError)?
      .ok_or(error::Error::CannotParseTweet { tweet })
  }

  pub async fn translate_boss_name(&self, raid_boss: RaidBoss) -> Result<String> {
    let (send, recv) = oneshot::channel();
    let msg = ActorMessage::TranslateBossName {
      raid_boss: raid_boss.clone(),
      respond_to: send,
    };
    let _ = self.sender.send(msg).await;
    recv
      .await
      .map_err(|_| error::Error::SenderSendError)?
      .ok_or(error::Error::CannotTranslateError {
        name: raid_boss.get_boss_name().into(),
      })
  }
}
