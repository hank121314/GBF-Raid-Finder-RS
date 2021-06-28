use crate::{common::redis::gbf_persistence_raid_tweets_keys, error, server::state::AppState};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct GetPersistenceBossRequest {
  pub boss_names: Vec<String>,
}

pub async fn get_persistence_boss(
  request: GetPersistenceBossRequest,
  app_state: AppState,
) -> Result<impl warp::Reply, warp::Rejection> {
  let boss_names = request.boss_names;
  let mut response = HashMap::new();

  for boss_name in boss_names.iter() {
    let persistence_keys = app_state
      .redis
      .keys(gbf_persistence_raid_tweets_keys(boss_name))
      .await
      .map_err(|_| error::HttpError::CannotGetRedisKeysError.new())?;
    let tweets_bytes: Vec<Vec<u8>> = app_state
      .redis
      .mget_protobuf_raw(persistence_keys)
      .await
      .map_err(|_| error::HttpError::CannotGetRedisKeysError.new())?;
    response.insert(boss_name, tweets_bytes);
  }

  Ok(warp::reply::json(&response))
}
