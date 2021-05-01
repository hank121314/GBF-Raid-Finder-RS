use crate::{error::Error, Result};

use std::env;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct Config<'a> {
  pub api_key: Cow<'a, str>,
  pub api_secret_key: Cow<'a, str>,
  pub access_token: Cow<'a, str>,
  pub access_token_secret: Cow<'a, str>,
}

impl<'a> Config<'a> {
  pub fn new() -> Result<Config<'a>> {
    let api_key = env::var("TWITTER_API_KEY").map_err(|_| Error::ApiKeyNotFound)?;
    let api_secret_key = env::var("TWITTER_API_SECRET_KEY").map_err(|_| Error::ApiSecretKeyNotFound)?;
    let access_token = env::var("TWITTER_ACCESS_TOKEN").map_err(|_| Error::AccessTokenNotFound)?;
    let access_token_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET").map_err(|_| Error::AccessTokenSecretNotFound)?;

    Ok(Config {
      api_key: Cow::Owned(api_key),
      api_secret_key: Cow::Owned(api_secret_key),
      access_token: Cow::Owned(access_token),
      access_token_secret: Cow::Owned(access_token_secret),
    })
  }
}
