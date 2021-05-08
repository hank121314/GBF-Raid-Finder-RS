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
  #[snafu(display("Cannot get redis keys, error: {}", error))]
  RedisGetKeysError { error: redis::RedisError },
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
  CannotGetStream { error: isahc::error::Error },
  #[snafu(display("Http request get bad response."))]
  BadResponseError,
  #[snafu(display("Invalid http method"))]
  InvalidHttpMethod,
  #[snafu(display("Cannot build request"))]
  CannotBuildRequest,
  #[snafu(display("Sender cannot send the request"))]
  SenderSendError,
  #[snafu(display("Unexpect EOF."))]
  StreamEOFError,
  #[snafu(display("IO Error, cannot read response, error: {}", error))]
  IOError { error: std::io::Error },

  /// Image Comparison Error
  #[snafu(display("Cannot get image from url, error: {}", error))]
  ImageCannotGetError { error: reqwest::Error },
  #[snafu(display("Cannot get response with bytes, error: {}", error))]
  BytesParseImageError { error: reqwest::Error },
  #[snafu(display("Cannot get image from bytes, error: {}", error))]
  ImageParseBytesError { error: load_image::Error },
  #[snafu(display("Cannot convert this image to image data"))]
  ImageToImageDataError,

  /// Parse Error
  #[snafu(display("Cannot parse this tweet: {:?}", tweet))]
  CannotParseTweet { tweet: crate::models::Tweet },
  #[snafu(display("JSON parse error, error: {}", error))]
  JSONParseError { error: serde_json::Error },
  #[snafu(display("Protobuf parse error, error: {}", error))]
  ProtobufParseError { error: protobuf::ProtobufError },
  #[snafu(display("Protobuf write to bytes parse error, error: {}", error))]
  ProtobufWriteError { error: protobuf::ProtobufError },
  #[snafu(display("Cannot parse HashMap to String, error: {}", error))]
  CannotParseHashMapError { error: serde_json::Error },
  #[snafu(display("Cannot parse u32 to usize"))]
  U32ToUSizeError,

  /// Logger Error
  #[snafu(display("Can not create logger"))]
  CannotCreateLogger,

  /// Common Error
  #[snafu(display("Tokio runtime error"))]
  TokioRuntimeError,
  #[snafu(display("Tokio translator runtime error"))]
  TokioTranslatorRuntimeError,
  #[snafu(display("Cannot translate given name, name: {}", name))]
  CannotTranslateError { name: String },
  #[snafu(display("String parse from bytes error, error: {}", error))]
  StringParseFromBytesError { error: std::string::FromUtf8Error },
  #[snafu(display("Future already complete without streaming"))]
  FutureAlreadyCompleted,
}
