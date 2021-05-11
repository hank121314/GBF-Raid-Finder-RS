mod client;
mod common;
mod config;
mod error;
mod image;
mod logger;
mod models;
mod parsers;
mod proto;
mod resources;
mod tasks;

use crate::{
  client::redis::Redis,
  client::{filter_stream::StreamingSource, http::FilterStreamClient},
  common::redis::GBF_TRANSLATOR_KEY,
  config::Config,
  error::Error,
  models::Tweet,
  resources::STREAM_URL,
};
use futures::StreamExt;
use futures_retry::{FutureRetry, RetryPolicy};
use log::{error as log_error, info};
use std::sync::Arc;
use tasks::TweetActorHandle;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
pub async fn main() -> Result<()> {
  logger::create_logger("/var/log/", "raid-finder-stream", 3)?;

  let config = Config::new()?;

  // Create twitter filter stream client
  let filter_stream_client = FilterStreamClient::new(
    config,
    vec!["参加者募集！", ":参戦ID", "I need backup!", ":Battle ID"],
    "true",
  );

  // Create tweet handler
  let redis = Redis::new("redis://127.0.0.1/")?;

  let map = redis.get_hash_map(GBF_TRANSLATOR_KEY).await?;

  let tweet_handler = Arc::new(TweetActorHandle::new(redis, map));

  FutureRetry::new(
    || async {
      let mut stream: StreamingSource<Tweet> = filter_stream_client.oauth_stream(STREAM_URL).await?;

      while let Some(data) = stream.next().await {
        let tweet = data?;
        if let Some((raid_boss, mut raid_tweet)) = tweet_handler.parse_tweet(tweet).await? {
          let boss_name: String = raid_boss.get_boss_name().into();
          if let Some(translated_boss_name) = tweet_handler.translate_boss_name(raid_boss).await? {
            // We can infer from empty translated_boss_name that the task is pending
            if translated_boss_name.is_empty() {
              info!("Translating task of {} is pending...", boss_name);
            } else {
              raid_tweet.set_boss_name(translated_boss_name);
              tweet_handler.persist_raid_tweet(raid_tweet).await;
            }
          }
        }
      }

      Ok::<(), Error>(())
    },
    |e: error::Error| {
      return match e {
        error::Error::StreamEOFError => {
          info!("Get EOF in twitter stream api will restart in 5 second.");
          RetryPolicy::WaitRetry(std::time::Duration::from_secs(5))
        }
        error::Error::JSONParseError { error: _ } => {
          info!("JSON Parse error by given string, stream might be cut off. Will restart stream in 5 second.");
          RetryPolicy::WaitRetry(std::time::Duration::from_secs(5))
        }
        _ => {
          log_error!("Some error encounter, error: {:?}", e);
          RetryPolicy::ForwardError(e)
        }
      };
    },
  )
  .await
  .map(|result| result.0)
  .map_err(|error| error.0)
}
