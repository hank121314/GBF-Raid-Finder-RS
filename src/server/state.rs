use crate::{client::redis::Redis, FinderClients};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
  pub redis: Arc<Redis>,
  pub clients: FinderClients,
}

impl AppState {
  pub fn new(redis: Arc<Redis>, clients: FinderClients) -> Self {
    AppState { redis, clients }
  }
}
