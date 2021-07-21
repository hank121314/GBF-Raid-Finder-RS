use crate::{common::redis::gbf_persistence_raid_tweets_keys, error, server::state::AppState};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct GetPersistenceBossRequest {
  pub boss_names: Vec<String>,
  pub limit: u32,
}

/// 
/// Get raid tweet which persist in the database by its name
/// 
/// # Arguments
/// * `request` - A JSON object with key of boss_names which is a vector string.
/// 
pub async fn get_persistence_boss(
  request: GetPersistenceBossRequest,
  app_state: AppState,
) -> Result<impl warp::Reply, warp::Rejection> {
  let boss_names = request.boss_names;
  let limit = request.limit;
  let mut response = HashMap::new();

  for boss_name in boss_names.iter() {
    let mut persistence_keys = app_state
      .redis
      .keys(gbf_persistence_raid_tweets_keys(boss_name))
      .await
      .map_err(|_| error::HttpError::CannotGetRedisKeysError.reject())?;
    persistence_keys.sort_by(|a, b| {
      let last = |string: String| string.split('.').last().map(|str| str.to_owned());
      if let (Some(last_a), Some(last_b)) = (last(a.into()), last(b.into())) {
        return last_b.cmp(&last_a);
      }
      b.cmp(a)
    });
    persistence_keys.truncate(limit as usize);
    let tweets_bytes: Vec<Vec<u8>> = app_state
      .redis
      .mget_protobuf_raw(persistence_keys)
      .await
      .map_err(|_| error::HttpError::CannotGetRedisKeysError.reject())?;
    response.insert(boss_name, tweets_bytes);
  }

  Ok(warp::reply::json(&response))
}
