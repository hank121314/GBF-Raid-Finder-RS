use crate::{common::redis::gbf_raid_boss_keys, server::state::AppState, error};
use serde::Deserialize;
use warp::hyper::StatusCode;

#[derive(Deserialize, Clone, Copy)]
pub struct GetBossRequest {
  pub level: u32,
}

pub async fn get_bosses(request: GetBossRequest, app_state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
  let level = request.level;

  let boss_keys = app_state
    .redis
    .keys(gbf_raid_boss_keys(level))
    .await
    .map_err(|_| error::HttpError::CannotGetRedisKeysError.reject())?;

  let bosses: Vec<Vec<u8>> = app_state
    .redis
    .mget_protobuf_raw(boss_keys)
    .await
    .map_err(|_| error::HttpError::CannotMGetRedisError.reject())?;

  Ok(warp::reply::with_status(warp::reply::json(&bosses), StatusCode::OK))
}
