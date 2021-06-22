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

use crate::models::TranslatorResult;
use crate::{
  client::redis::Redis,
  client::{filter_stream::StreamingSource, http::FilterStreamClient},
  common::redis::get_translator_map,
  config::Config,
  models::{Language, Tweet},
  proto::raid_finder::raid_finder_server::RaidFinderServer,
  resources::http::STREAM_URL,
  tasks::g_rpc::StreamingService,
};
use futures::{StreamExt, TryStreamExt};
use futures_retry::{FutureRetry, RetryPolicy};
use log::{error as log_error, info};
use std::{collections::HashMap, str::FromStr, sync::Arc, env};
use tasks::tweet::TweetActorHandle;
use tonic::transport::Server;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
pub async fn main() -> Result<()> {
  let log_path = env::var("GBF_RAID_FINDER_LOG_PATH").unwrap_or_else(|_| "/var/log".to_owned());
  
  logger::create_logger(log_path, "raid-finder-stream", 3)?;

  let redis_url = env::var("REDIS_URL").map_err(|_| error::Error::RedisURLNotFound)?;

  let config = Config::new()?;

  // Create twitter filter stream client
  let filter_stream_client = FilterStreamClient::new(
    config,
    vec!["参加者募集！", ":参戦ID", "I need backup!", ":Battle ID"],
    "true",
  );

  // Create tweet handler
  let singleton_redis = Redis::new(redis_url)?;

  let redis = Arc::new(singleton_redis);

  let service_redis = redis.clone();

  // Initialize translator map with redis keys `gbf:translator:*`
  let map = get_translator_map(&redis).await.unwrap_or_else(|_| HashMap::new());

  let tweet_handler = TweetActorHandle::new(redis, map);

  // Sender between `StreamingSource` and gRPC Server
  let (tweet_sender, tweet_receiver) = crossbeam_channel::bounded(1024);

  // Create gRPC server
  tokio::spawn(async move {
    let service = StreamingService::new(service_redis, tweet_receiver);
    let addr = "0.0.0.0:50051".parse().unwrap();

    info!("gRPC server listening on {}...", addr);

    Server::builder()
      .add_service(RaidFinderServer::new(service))
      .serve(addr)
      .await
      .map_err(|error| error::Error::CannotStartGRPCServer { error })?;

    Ok::<(), error::Error>(())
  });

  FutureRetry::new(
    || async {
      let stream: StreamingSource<Tweet> = filter_stream_client.oauth_stream(STREAM_URL).await?;

      let tweet_stream = stream
        .and_then(|tweet| async {
          tweet_handler
            .parse_tweet(tweet.clone())
            .await
            .ok_or(error::Error::CannotParseTweet { tweet })
        })
        .and_then(|(raid_boss_raw, mut raid_tweet)| async {
          tweet_handler
            .translate_boss_name(raid_boss_raw.clone())
            .await
            .and_then(|translator_result| match translator_result {
              TranslatorResult::Pending => None,
              TranslatorResult::Success {
                result: translated_name,
              } => {
                let language = Language::from_str(raid_boss_raw.get_language()).unwrap();
                if language == Language::English {
                  raid_tweet.set_boss_name(translated_name.to_owned());
                }

                Some(raid_tweet)
              }
            })
            .ok_or(error::Error::CannotTranslateError {
              name: raid_boss_raw.boss_name,
            })
        })
        .and_then(|raid_tweet| async {
          tweet_handler.persist_raid_tweet(raid_tweet.clone()).await;

          Ok(raid_tweet)
        });

      // Calls to async fn return anonymous Future values that are !Unpin. These values must be pinned before they can be polled.
      tokio::pin!(tweet_stream);

      while let Some(chunk) = tweet_stream.next().await {
        let raid_tweet = match chunk {
          Ok(tweet) => tweet,
          // If the error occur means this tweet stream failed in and_then combinators.
          Err(_) => continue,
        };
        tweet_sender
          .send(raid_tweet.clone())
          .map_err(|_| error::Error::SenderSendError)?;
      }

      Err::<(), error::Error>(error::Error::StreamEOFError)
    },
    |e: error::Error| match e {
      error::Error::StreamEOFError => {
        info!("Get EOF in twitter stream api will restart in 5 second.");
        RetryPolicy::WaitRetry(std::time::Duration::from_secs(5))
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
