use crate::{error, Result};

use std::env;

#[derive(Clone, Debug)]
pub struct Config {
  pub api_key: String,
  pub api_secret_key: String,
  pub access_token: String,
  pub access_token_secret: String,
  pub redis_url: String,
  pub log_path: String,
}

impl Config {
  pub fn new() -> Result<Config> {
    let api_key = env::var("TWITTER_API_KEY").map_err(|_| error::Error::ApiKeyNotFound)?;
    let api_secret_key = env::var("TWITTER_API_SECRET_KEY").map_err(|_| error::Error::ApiSecretKeyNotFound)?;
    let access_token = env::var("TWITTER_ACCESS_TOKEN").map_err(|_| error::Error::AccessTokenNotFound)?;
    let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET").map_err(|_| error::Error::AccessTokenSecretNotFound)?;
    let redis_url = env::var("REDIS_URL").map_err(|_| error::Error::RedisURLNotFound)?;
    let log_path = env::var("GBF_RAID_FINDER_LOG_PATH").unwrap_or_else(|_| "/var/log".to_owned());

    Ok(Config {
      api_key,
      api_secret_key,
      access_token,
      access_token_secret,
      redis_url,
      log_path,
    })
  }
}
