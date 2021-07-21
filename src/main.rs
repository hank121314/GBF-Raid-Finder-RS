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
mod server;
mod tasks;

use crate::{
  client::redis::Redis,
  client::{filter_stream::StreamingSource, http::FilterStreamClient},
  common::redis::get_translator_map,
  config::Config,
  models::{TranslatorResult, Tweet},
  proto::{raid_boss_raw::RaidBossRaw, raid_tweet::RaidTweet},
  resources::http::STREAM_URL,
  server::{client::FinderClient, http::create_http_server},
  tasks::tweet::TweetActorHandle,
};
use futures::{FutureExt, TryStreamExt};
use futures_retry::{FutureRetry, RetryPolicy};
use log::{error as log_error, info};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

pub type FinderClients = Arc<RwLock<HashMap<String, FinderClient>>>;
pub type Result<T, E = error::Error> = std::result::Result<T, E>;

#[tokio::main]
pub async fn main() -> Result<()> {
  let config = Config::new()?;

  logger::create_logger(config.log_path.as_str(), "raid-finder-stream", 3)?;

  // Create redis client
  let redis = Redis::new(config.redis_url.as_str())?;

  let redis = Arc::new(redis);

  // Create twitter filter stream client
  let filter_stream_client = FilterStreamClient::new(
    config,
    vec!["参加者募集！", ":参戦ID", "I need backup!", ":Battle ID"],
    "true",
  );

  // Initialize translator map with redis keys `gbf:translator:*`
  let translator_map = get_translator_map(&redis).await.unwrap_or_else(|_| HashMap::new());

  // Create tweet handler to consuming incoming stream
  let tweet_handler = TweetActorHandle::new(redis.clone(), translator_map);

  // Create an empty client map
  let finder_clients: FinderClients = Arc::new(RwLock::new(HashMap::new()));

  // Create http/ws server
  create_http_server(redis, finder_clients.clone());

  FutureRetry::new(
    || async {
      // Get tweet stream source from STREAM_URL
      let stream: StreamingSource<Tweet> = filter_stream_client.oauth_stream(STREAM_URL).await?;

      let tweet_stream = stream
        .and_then(|tweet| tweet_handler.parse_tweet(tweet))
        .and_then(|(raid_boss_raw, raid_tweet)| {
          tweet_handler
            .translate_boss_name(raid_boss_raw.clone())
            .map(|result| Ok((raid_boss_raw, raid_tweet, result)))
        })
        .and_then(
          |(raid_boss_raw, raid_tweet, translator_result): (RaidBossRaw, RaidTweet, TranslatorResult)| {
            tweet_handler.translate_tweet(raid_boss_raw, raid_tweet, translator_result)
          },
        )
        .and_then(|raid_tweet| tweet_handler.persist_raid_tweet(raid_tweet))
        .timeout(std::time::Duration::new(5, 0));

      // Calls to async fn return anonymous Future values that are !Unpin. These values must be pinned before they can be polled.
      tokio::pin!(tweet_stream);

      while let Some(Ok(chunk)) = tweet_stream.next().await {
        match chunk {
          Ok(raid_tweet) => {
            tasks::websocket::sending_message_to_websocket_client(raid_tweet, finder_clients.clone());
          }
          // Only if we get StreamUnexpectedError/StreamEOFError/BadResponseError should reconnect the stream.
          // Otherwise we will skip the tweet.
          Err(stream_error) => match stream_error {
            error::Error::StreamUnexpectedError => return Err(stream_error),
            error::Error::StreamEOFError => return Err(stream_error),
            error::Error::BadResponseError => return Err(stream_error),
            _ => continue,
          },
        };
      }

      Err::<(), error::Error>(error::Error::StreamUnexpectedError)
    },
    |e: error::Error| match e {
      error::Error::StreamUnexpectedError => {
        info!("Get unexpected error while streaming tweets will restart in 5 second.");
        RetryPolicy::WaitRetry(std::time::Duration::from_secs(5))
      }
      error::Error::BadResponseError => {
        info!("Get bad response when connecting to twitter stream api will restart in 5 second.");
        RetryPolicy::WaitRetry(std::time::Duration::from_secs(5))
      }
      error::Error::StreamEOFError => {
        info!("Get EOF in twitter stream api will restart in 1 second.");
        RetryPolicy::WaitRetry(std::time::Duration::from_secs(1))
      }
      _ => {
        log_error!("Some error encounter, error: {:?}", e);
        RetryPolicy::ForwardError(e)
      }
    },
  )
  .await
  .map(|result| result.0)
  .map_err(|error| error.0)
}
