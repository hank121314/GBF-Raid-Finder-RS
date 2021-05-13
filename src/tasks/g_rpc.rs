use crate::{
  client::redis::Redis,
  common::redis::{gbf_persistence_raid_tweets_key, gbf_raid_boss_keys},
  error,
  proto::{
    raid_boss::RaidBoss,
    raid_finder::{
      raid_finder_server::RaidFinder, GetBossRequest, GetBossResponse, GetPersistenceBossRequest,
      GetPersistenceBossResponse, StreamRequest,
    },
    raid_tweet::RaidTweet,
  },
};
use std::sync::Arc;
use crossbeam_channel::Receiver;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

pub struct StreamingService {
  redis: Arc<Redis>,
  receiver: Receiver<RaidTweet>,
}

impl StreamingService {
  pub fn new(redis: Arc<Redis>, receiver: Receiver<RaidTweet>) -> Self {
    Self { redis, receiver }
  }
}

#[tonic::async_trait]
impl RaidFinder for StreamingService {
  async fn get_bosses(&self, request: Request<GetBossRequest>) -> Result<Response<GetBossResponse>, Status> {
    let level = request.into_inner().level;

    let boss_keys = self
      .redis
      .keys(gbf_raid_boss_keys(level))
      .await
      .map_err(|_| error::GrpcError::CannotGetRedisKeysError.new())?;

    let bosses: Vec<RaidBoss> = self
      .redis
      .mget_protobuf(boss_keys)
      .await
      .map_err(|_| error::GrpcError::CannotMGetRedisError.new())?;

    let response = GetBossResponse { bosses };

    Ok(Response::new(response))
  }

  async fn get_persistence_boss(
    &self,
    request: Request<GetPersistenceBossRequest>,
  ) -> Result<Response<GetPersistenceBossResponse>, Status> {
    let req = request.into_inner();
    let boss_name = req.boss_name;
    let persistence_keys = self
      .redis
      .keys(gbf_persistence_raid_tweets_key(boss_name))
      .await
      .map_err(|_| error::GrpcError::CannotGetRedisKeysError.new())?;
    let tweets: Vec<RaidTweet> = self
      .redis
      .mget_protobuf(persistence_keys)
      .await
      .map_err(|_| error::GrpcError::CannotMGetRedisError.new())?;

    let response = GetPersistenceBossResponse { tweets };

    Ok(Response::new(response))
  }

  type StartStreamStream = ReceiverStream<Result<RaidTweet, Status>>;

  async fn start_stream(&self, request: Request<StreamRequest>) -> Result<Response<Self::StartStreamStream>, Status> {
    let req = request.into_inner();
    let (tx, rx) = mpsc::channel(4);
    let receiver = self.receiver.clone();
    let bosses = req.boss_names;

    tokio::spawn(async move {
      while let Ok(tweet) = receiver.recv() {
        if bosses.contains(&tweet.boss_name) {
          tx.send(Ok(tweet.clone()))
            .await
            .map_err(|_| error::Error::SenderSendError)?;
        }
      }
      Ok::<(), error::Error>(())
    });

    Ok(Response::new(ReceiverStream::new(rx)))
  }
}
