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

use crate::proto::raid_finder::raid_finder_server::RaidFinderServer;
use crate::tasks::g_rpc::StreamingService;
use crate::{
  client::redis::Redis,
  client::{filter_stream::StreamingSource, http::FilterStreamClient},
  common::redis::{get_translator_map, GBF_TRANSLATOR_KEY},
  config::Config,
  error::Error,
  models::{Language, Tweet},
  resources::STREAM_URL,
};
use futures::StreamExt;
use futures_retry::{FutureRetry, RetryPolicy};
use log::{error as log_error, info};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tasks::tweet::TweetActorHandle;
use tonic::transport::Server;

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
  let singleton_redis = Redis::new("redis://127.0.0.1/")?;

  let redis = Arc::new(singleton_redis);

  let service_redis = redis.clone();

  let search_keys = format!("{}:*", GBF_TRANSLATOR_KEY);

  let mut map = get_translator_map(&redis, search_keys)
    .await
    .unwrap_or_else(|_| HashMap::new());

  let replace = format!("{}:", GBF_TRANSLATOR_KEY);

  map = map.into_iter().map(|k| (k.0.replace(&replace, ""), k.1)).collect();

  let tweet_handler = Arc::new(TweetActorHandle::new(redis, map));

  let (tweet_sender, tweet_receiver) = crossbeam_channel::bounded(1024);

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
      let mut stream: StreamingSource<Tweet> = filter_stream_client.oauth_stream(STREAM_URL).await?;
      while let Some(data) = stream.next().await {
        let tweet = data?;
        let (raid_boss, mut raid_tweet) = match tweet_handler.parse_tweet(tweet).await? {
          Some((b, t)) => (b, t),
          None => continue,
        };
        let boss_name: String = raid_boss.get_boss_name().into();
        let language = Language::from_str(raid_boss.get_language()).unwrap();
        let translated_boss_name = match tweet_handler.translate_boss_name(raid_boss).await? {
          Some(t) => t,
          None => continue,
        };
        // We can infer from empty translated_boss_name that the task is pending
        match translated_boss_name.is_empty() {
          true => {
            info!("Translating task of {} is pending...", boss_name);
          }
          false => {
            if language == Language::English {
              raid_tweet.set_boss_name(translated_boss_name);
            }
            tweet_sender
              .send(raid_tweet.clone())
              .map_err(|_| error::Error::SenderSendError)?;
            tweet_handler.persist_raid_tweet(raid_tweet).await;
          }
        };
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
