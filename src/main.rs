mod client;
mod common;
mod config;
mod error;
mod models;
mod parsers;
mod proto;
mod resources;

use crate::{
  client::filter_stream::FilterStreamClient, client::redis::Redis, config::Config, models::Tweet,
  common::gbf_redis_key,
  parsers::status::StatusParser, resources::BOSS_EXPIRE_IN_30_DAYS_TTL,
};
use tokio::sync::mpsc::channel;
use std::borrow::Borrow;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

#[tokio::main]
pub async fn main() -> Result<()> {
  let (tx, mut rx) = channel(1024);
  let redis = Redis::new("redis://127.0.0.1/")?;
  let config = Config::new()?;
  let filter_stream_client = FilterStreamClient::new(
    config,
    vec!["参加者募集！", ":参戦ID", "I need backup!", ":Battle ID"],
    "true",
  );

  let consuming_raid_tweets = async move {
    while let Some(tweet) = rx.recv().await {
      if let Some(raid) = StatusParser::parse(tweet) {
        let redis_key = gbf_redis_key(raid.borrow());
        redis.set_protobuf(&redis_key, raid, BOSS_EXPIRE_IN_30_DAYS_TTL).await?;
      }
    }
    Ok::<(), error::Error>(())
  };

  tokio::spawn(consuming_raid_tweets);

  filter_stream_client.stream::<Tweet>(tx).await?;
  Ok(())
}
