use crate::{error::Error, Result};

use std::env;

#[derive(Clone, Debug)]
pub struct Config {
  pub api_key: String,
  pub api_secret_key: String,
  pub access_token: String,
  pub access_token_secret:String,
}

impl Config {
  pub fn new() -> Result<Config> {
    let api_key = env::var("TWITTER_API_KEY").map_err(|_| Error::ApiKeyNotFound)?;
    let api_secret_key = env::var("TWITTER_API_SECRET_KEY").map_err(|_| Error::ApiSecretKeyNotFound)?;
    let access_token = env::var("TWITTER_ACCESS_TOKEN").map_err(|_| Error::AccessTokenNotFound)?;
    let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET").map_err(|_| Error::AccessTokenSecretNotFound)?;

    Ok(Config {
      api_key,
      api_secret_key,
      access_token,
      access_token_secret,
    })
  }
}
