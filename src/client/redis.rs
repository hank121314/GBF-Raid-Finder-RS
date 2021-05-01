use crate::{error::Error, Result};
use redis::{AsyncCommands, Client};
use std::{convert::TryInto, sync::Arc};

#[derive(Clone)]
pub struct Redis {
  client: Arc<Client>,
}

impl Redis {
  pub fn new<S>(address: S) -> Result<Self>
  where
    S: Into<String>,
  {
    let client = redis::Client::open(address.into()).map_err(|error| Error::RedisConnectionError { error })?;
    Ok(Redis {
      client: Arc::new(client),
    })
  }

  pub async fn set_protobuf<S, T, U>(&self, key: S, value: T, ttl: U) -> Result<()>
  where
    S: Into<String>,
    T: protobuf::Message + Send,
    U: TryInto<usize> + Copy,
  {
    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| Error::RedisGetConnectionError { error })?;
    let bytes = value
      .write_to_bytes()
      .map_err(|error| Error::ProtobufWriteError { error })?;

    let redis_key = key.into();

    connection
      .set(&redis_key, bytes)
      .await
      .map_err(|error| Error::RedisSetValueError { error })?;

    if let Ok(redis_ttl) = ttl.try_into() {
      if redis_ttl != 0 {
        connection
          .expire(&redis_key, redis_ttl)
          .await
          .map_err(|error| Error::RedisExpireError { error })?;
      }
    }

    Ok(())
  }

  pub async fn get_protobuf<T, S>(&self, key: S) -> Result<T>
  where
    S: Into<String>,
    T: protobuf::Message + Send,
  {
    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| Error::RedisGetConnectionError { error })?;
    let bytes: Vec<u8> = connection
      .get(key.into())
      .await
      .map_err(|error| Error::RedisGetValueError { error })?;

    T::parse_from_bytes(&bytes).map_err(|error| Error::ProtobufParseError { error })
  }
}
