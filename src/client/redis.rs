use crate::{error, Result};
use redis::{AsyncCommands, Client};
use std::convert::TryInto;

#[derive(Clone)]
pub struct Redis {
  client: Client,
}

#[allow(dead_code)]
impl Redis {
  pub fn new<S>(address: S) -> Result<Self>
  where
    S: Into<String>,
  {
    let client = redis::Client::open(address.into()).map_err(|error| error::Error::RedisConnection { error })?;
    Ok(Redis { client })
  }

  pub async fn get_protobuf<T, S>(&self, key: S) -> Result<T>
  where
    S: Into<String>,
    T: prost::Message + std::default::Default,
  {
    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnection { error })?;
    let bytes: Vec<u8> = connection
      .get(key.into())
      .await
      .map_err(|error| error::Error::RedisGetValue { error })?;

    T::decode(&mut bytes.as_slice()).map_err(|error| error::Error::ProtobufParse { error })
  }

  pub async fn mget_protobuf_raw<V, S>(&self, keys: V) -> Result<Vec<Vec<u8>>>
  where
    S: Into<String>,
    V: IntoIterator<Item = S>,
  {
    let redis_keys = keys.into_iter().map(|key| key.into()).collect::<Vec<String>>();

    if redis_keys.is_empty() {
      return Ok(vec![]);
    }

    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnection { error })?;

    let bytes: Vec<Vec<u8>> = connection
      .get(redis_keys)
      .await
      .map_err(|error| error::Error::RedisGetValue { error })?;

    Ok(bytes)
  }

  pub async fn mget_protobuf<T, V, S>(&self, keys: V) -> Result<Vec<T>>
  where
    S: Into<String>,
    V: IntoIterator<Item = S>,
    T: prost::Message + std::default::Default,
  {
    let redis_keys = keys.into_iter().map(|key| key.into()).collect::<Vec<String>>();

    if redis_keys.is_empty() {
      return Ok(vec![]);
    }

    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnection { error })?;

    let bytes: Vec<Vec<u8>> = connection
      .get(redis_keys)
      .await
      .map_err(|error| error::Error::RedisGetValue { error })?;

    bytes
      .into_iter()
      .map(|ref byte| T::decode(&mut byte.as_slice()).map_err(|error| error::Error::ProtobufParse { error }))
      .collect()
  }

  pub async fn mget_string<S, I>(&self, keys: I) -> Result<Vec<String>>
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnection { error })?;

    connection
      .get(keys.into_iter().map(|k| k.into()).collect::<Vec<_>>())
      .await
      .map_err(|error| error::Error::RedisGetValue { error })
  }

  pub async fn get_string<S, I>(&self, key: I) -> Result<String>
  where
    I: Into<String>,
    S: Into<String>,
  {
    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnection { error })?;

    connection
      .get(key.into())
      .await
      .map_err(|error| error::Error::RedisGetValue { error })
  }

  pub async fn set_protobuf<S, T, U>(&self, key: S, value: T, ttl: U) -> Result<()>
  where
    S: Into<String>,
    T: prost::Message + std::default::Default,
    U: TryInto<usize> + Copy,
  {
    let redis_key = key.into();
    let redis_ttl = ttl.try_into().map_err(|_| error::Error::U32ToUSize)?;

    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnection { error })?;
    let mut bytes = Vec::new();

    value
      .encode(&mut bytes)
      .map_err(|error| error::Error::ProtobufWrite { error })?;

    connection
      .set(&redis_key, bytes)
      .await
      .map_err(|error| error::Error::RedisSetValue { error })?;

    if redis_ttl != 0 {
      connection
        .expire(&redis_key, redis_ttl)
        .await
        .map_err(|error| error::Error::RedisExpire { error })?;
    }

    Ok(())
  }

  pub async fn expire<S, U>(&self, key: S, ttl: U) -> Result<()>
  where
    S: Into<String>,
    U: TryInto<usize> + Copy,
  {
    let redis_key = key.into();
    let redis_ttl = ttl.try_into().map_err(|_| error::Error::U32ToUSize)?;

    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnection { error })?;

    if redis_ttl != 0 {
      connection
        .expire(&redis_key, redis_ttl)
        .await
        .map_err(|error| error::Error::RedisExpire { error })?;
    }

    Ok(())
  }

  pub async fn set_multiple_string<I, K, V>(&self, value: I) -> Result<()>
  where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
  {
    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnection { error })?;

    connection
      .set_multiple(
        value
          .into_iter()
          .map(|v| (v.0.into(), v.1.into()))
          .collect::<Vec<_>>()
          .as_slice(),
      )
      .await
      .map_err(|error| error::Error::RedisSetValue { error })?;

    Ok(())
  }

  pub async fn keys<S>(&self, key: S) -> Result<Vec<String>>
  where
    S: Into<String>,
  {
    let redis_key = key.into();

    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnection { error })?;

    connection
      .keys::<String, Vec<String>>(redis_key)
      .await
      .map_err(|error| error::Error::RedisGetKeys { error })
  }
}
