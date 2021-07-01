use crate::{client::redis::Redis, common::chrono::current_timestamp_u64, FinderClients};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

#[derive(Clone)]
pub struct AppState {
  pub redis: Arc<Redis>,
  pub clients: FinderClients,
  pub health_check: Arc<AtomicU64>,
}

impl AppState {
  pub fn new(redis: Arc<Redis>, clients: FinderClients) -> Self {
    AppState {
      redis,
      clients,
      health_check: Arc::new(AtomicU64::new(current_timestamp_u64())),
    }
  }
}
