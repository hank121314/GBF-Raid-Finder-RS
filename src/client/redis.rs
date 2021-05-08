use crate::{error, Result};
use redis::{AsyncCommands, Client};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
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
    let client = redis::Client::open(address.into()).map_err(|error| error::Error::RedisConnectionError { error })?;
    Ok(Redis {
      client: Arc::new(client),
    })
  }

  pub async fn get_protobuf<T, S>(&self, key: S) -> Result<T>
  where
    S: Into<String>,
    T: protobuf::Message,
  {
    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnectionError { error })?;
    let bytes: Vec<u8> = connection
      .get(key.into())
      .await
      .map_err(|error| error::Error::RedisGetValueError { error })?;

    T::parse_from_bytes(&bytes).map_err(|error| error::Error::ProtobufParseError { error })
  }

  pub async fn mget_protobuf<T, V, S>(&self, keys: V) -> Result<Vec<T>>
  where
    S: Into<String>,
    V: IntoIterator<Item = S>,
    T: protobuf::Message,
  {
    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnectionError { error })?;
    let redis_keys = keys.into_iter().map(|key| key.into()).collect::<Vec<String>>();
    let bytes: Vec<Vec<u8>> = connection
      .get(redis_keys)
      .await
      .map_err(|error| error::Error::RedisGetValueError { error })?;

    bytes
      .into_iter()
      .map(|ref byte| T::parse_from_bytes(byte).map_err(|error| error::Error::ProtobufParseError { error }))
      .collect()
  }

  pub async fn get_hash_map<S, K, V>(&self, key: S) -> Result<HashMap<K, V>>
  where
    S: Into<String>,
    K: Into<String> + DeserializeOwned + std::cmp::Eq + std::hash::Hash,
    V: DeserializeOwned,
  {
    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnectionError { error })?;
    let string: String = connection.get(key.into()).await.unwrap_or("{}".into());

    serde_json::from_str::<HashMap<K, V>>(string.as_ref()).map_err(|error| error::Error::JSONParseError { error })
  }

  pub async fn set_protobuf<S, T, U>(&self, key: S, value: T, ttl: U) -> Result<()>
  where
    S: Into<String>,
    T: protobuf::Message,
    U: TryInto<usize> + Copy,
  {
    let redis_key = key.into();
    let redis_ttl = ttl.try_into().map_err(|_| error::Error::U32ToUSizeError)?;

    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnectionError { error })?;
    let bytes = value
      .write_to_bytes()
      .map_err(|error| error::Error::ProtobufWriteError { error })?;

    connection
      .set(&redis_key, bytes)
      .await
      .map_err(|error| error::Error::RedisSetValueError { error })?;

    if redis_ttl != 0 {
      connection
        .expire(&redis_key, redis_ttl)
        .await
        .map_err(|error| error::Error::RedisExpireError { error })?;
    }

    Ok(())
  }

  pub async fn set_hash_map<S, K, V, U>(&self, key: S, value: HashMap<K, V>, ttl: U) -> Result<()>
  where
    S: Into<String>,
    K: Into<String> + Serialize + std::cmp::Eq + std::hash::Hash,
    V: Serialize,
    U: TryInto<usize> + Copy,
  {
    let redis_key = key.into();
    let redis_ttl = ttl.try_into().map_err(|_| error::Error::U32ToUSizeError)?;

    let mut connection = self
      .client
      .get_tokio_connection()
      .await
      .map_err(|error| error::Error::RedisGetConnectionError { error })?;
    let string = serde_json::to_string(&value).map_err(|error| error::Error::CannotParseHashMapError { error })?;

    connection
      .set(&redis_key, string)
      .await
      .map_err(|error| error::Error::RedisSetValueError { error })?;

    if redis_ttl != 0 {
      connection
        .expire(&redis_key, redis_ttl)
        .await
        .map_err(|error| error::Error::RedisExpireError { error })?;
    }

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
      .map_err(|error| error::Error::RedisGetConnectionError { error })?;

    connection
      .keys::<String, Vec<String>>(redis_key)
      .await
      .map_err(|error| error::Error::RedisGetKeysError { error })
  }
}
