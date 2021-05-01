use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
  /// Environment Not Found
  #[snafu(display("Cannot find environment variable TWITTER_ACCESS_TOKEN"))]
  AccessTokenNotFound,
  #[snafu(display("Cannot find environment variable TWITTER_ACCESS_TOKEN_SECRET"))]
  AccessTokenSecretNotFound,
  #[snafu(display("Cannot find environment variable TWITTER_BEARER_TOKEN"))]
  BearerTokenNotFound,
  #[snafu(display("Cannot find environment variable TWITTER_API_KEY"))]
  ApiKeyNotFound,
  #[snafu(display("Cannot find environment variable TWITTER_API_SECRET_KEY"))]
  ApiSecretKeyNotFound,

  /// Redis Error
  #[snafu(display("Cannot get redis connection, error: {}", error))]
  RedisGetConnectionError { error: redis::RedisError },
  #[snafu(display("Cannot get redis value, error: {}", error))]
  RedisGetValueError { error: redis::RedisError },
  #[snafu(display("Cannot set redis value, error: {}", error))]
  RedisSetValueError { error: redis::RedisError },
  #[snafu(display("Cannot set redis expire, error: {}", error))]
  RedisExpireError { error: redis::RedisError },
  #[snafu(display("Cannot open redis connection, error: {}", error))]
  RedisConnectionError { error: redis::RedisError },

  /// HTTP Request Error
  #[snafu(display("Cannot get stream, error: {}", error))]
  CannotGetStream { error: reqwest::Error },
  #[snafu(display("Cannot build request"))]
  CannotBuildRequest,
  #[snafu(display("Sender cannot send the request"))]
  SenderSendError,

  /// Parse Error
  #[snafu(display("Bytes parse error, error: {}", error))]
  BytesParseError { error: reqwest::Error },
  #[snafu(display("JSON parse error, error: {}", error))]
  JSONParseError { error: serde_json::Error },
  #[snafu(display("Protobuf parse error, error: {}", error))]
  ProtobufParseError { error: protobuf::ProtobufError },
  #[snafu(display("Protobuf write to bytes parse error, error: {}", error))]
  ProtobufWriteError { error: protobuf::ProtobufError },
}
