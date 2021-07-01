use crate::{common::chrono::current_timestamp_u64, server::state::AppState};
use warp::hyper::StatusCode;
use std::sync::atomic::Ordering;

/// 
/// Health check service for kubernetes
/// should send a ping pack every 20 seconds or it will return an error.
/// 
pub fn healthz(app_state: AppState) -> impl warp::Reply {
  let now = current_timestamp_u64();
  let health_check = app_state.health_check.load(Ordering::Relaxed);
  let duration = now - health_check;
  app_state.health_check.store(now, Ordering::Relaxed);
  match duration > 20 {
    true => warp::reply::with_status(format!("error: {}", duration), StatusCode::INTERNAL_SERVER_ERROR),
    false => warp::reply::with_status("ok".to_owned(), StatusCode::OK),
  }
}
