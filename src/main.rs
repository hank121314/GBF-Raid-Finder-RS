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
  common::redis::{get_translator_map, GBF_TRANSLATOR_KEY},
  config::Config,
  models::{Language, Tweet},
  proto::raid_finder::raid_finder_server::RaidFinderServer,
  resources::STREAM_URL,
  tasks::g_rpc::StreamingService,
};
use futures::{StreamExt, TryStreamExt};
use futures_retry::{FutureRetry, RetryPolicy};
use log::{error as log_error, info};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tasks::tweet::TweetActorHandle;
use tonic::transport::Server;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

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
  let singleton_redis = Redis::new("redis://127.0.0.1/")?;

  let redis = Arc::new(singleton_redis);

  let service_redis = redis.clone();

  // Initialize translator map with redis keys `gbf:translator:*`
  let map = get_translator_map(&redis, format!("{}:*", GBF_TRANSLATOR_KEY))
    .await
    .unwrap_or_else(|_| HashMap::new());

  let tweet_handler = Arc::new(TweetActorHandle::new(redis, map));

  // Sender between `StreamingSource` and gRPC Server
  let (tweet_sender, tweet_receiver) = crossbeam_channel::bounded(1024);

  // Create gRPC server
  tokio::spawn(async move {
    let service = StreamingService::new(service_redis, tweet_receiver);
    let addr = "0.0.0.0:50052".parse().unwrap();

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
          match tweet_handler.parse_tweet(tweet.clone()).await? {
            Some(boss_and_tweet) => Ok(boss_and_tweet),
            None => Err(error::Error::CannotParseTweet { tweet }),
          }
        })
        .and_then(|(raid_boss_raw, raid_tweet)| async {
          match tweet_handler.translate_boss_name(raid_boss_raw.clone()).await? {
            Some(translated_name) => Ok((raid_boss_raw, raid_tweet, translated_name)),
            None => Err(error::Error::CannotTranslateError {
              name: raid_boss_raw.boss_name,
            }),
          }
        })
        .and_then(|(raid_boss_raw, mut raid_tweet, translated_name)| async {
          match translated_name.is_empty() {
            true => {
              info!("Translating task of {} is pending...", raid_boss_raw.get_boss_name());
              Err(error::Error::CannotTranslateError {
                name: raid_boss_raw.boss_name,
              })
            }
            false => {
              let language = Language::from_str(raid_boss_raw.get_language()).unwrap();
              if language == Language::English {
                raid_tweet.set_boss_name(translated_name);
              }

              tweet_handler.persist_raid_tweet(raid_tweet.clone()).await;

              Ok(raid_tweet)
            }
          }
        });

      tokio::pin!(tweet_stream);

      while let Some(chunk) = tweet_stream.next().await {
        let raid_tweet = match chunk {
          Ok(tweet) => tweet,
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
      error::Error::JSONParseError { error: _ } => {
        info!("JSON Parse error by given string, stream might be cut off. Will restart stream in 5 second.");
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
