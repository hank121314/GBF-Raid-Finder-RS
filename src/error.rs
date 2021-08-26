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
  #[snafu(display("Cannot find environment variable REDIS_URL"))]
  RedisURLNotFound,

  /// Redis Error
  #[snafu(display("Cannot get redis connection, error: {}", error))]
  RedisGetConnection { error: redis::RedisError },
  #[snafu(display("Cannot get redis keys, error: {}", error))]
  RedisGetKeys { error: redis::RedisError },
  #[snafu(display("Cannot get redis value, error: {}", error))]
  RedisGetValue { error: redis::RedisError },
  #[snafu(display("Cannot set redis value, error: {}", error))]
  RedisSetValue { error: redis::RedisError },
  #[snafu(display("Cannot set redis expire, error: {}", error))]
  RedisExpire { error: redis::RedisError },
  #[snafu(display("Cannot open redis connection, error: {}", error))]
  RedisConnection { error: redis::RedisError },

  /// HTTP Request Error
  #[snafu(display("Cannot get stream, error: {}", error))]
  CannotGetStream { error: hyper::Error },
  #[snafu(display("Http request get bad response."))]
  BadResponse,
  #[snafu(display("Invalid http method"))]
  InvalidHttpMethod,
  #[snafu(display("Cannot build request"))]
  CannotBuildRequest,
  #[snafu(display("Unexpect EOF."))]
  StreamEOF,
  #[snafu(display("Stream get unexpected error."))]
  StreamUnexpected,

  /// Websockets Error
  #[snafu(display("Websockets client error: {}", error))]
  WebsocketsClientError { error: warp::Error },
  #[snafu(display("Websockets client gone"))]
  WebsocketsClientClose,

  /// Image Comparison Error
  #[snafu(display("Cannot get image from url, error: {}", error))]
  ImageCannotGet { error: reqwest::Error },
  #[snafu(display("Cannot get response with bytes, error: {}", error))]
  BytesParseImage { error: reqwest::Error },
  #[snafu(display("Cannot get image from bytes, error: {}", error))]
  ImageParseBytes { error: load_image::Error },
  #[snafu(display("Cannot convert this image to image data"))]
  ImageToImageData,

  /// Parse Error
  #[snafu(display("Cannot parse this tweet: {:?}", tweet))]
  CannotParseTweet { tweet: crate::models::Tweet },
  #[snafu(display("JSON parse error, error: {}", error))]
  JSONParse { error: serde_json::Error },
  #[snafu(display("Protobuf parse error, error: {}", error))]
  ProtobufParse { error: prost::DecodeError },
  #[snafu(display("Protobuf write to bytes parse error, error: {}", error))]
  ProtobufWrite { error: prost::EncodeError },
  #[snafu(display("Cannot parse u32 to usize"))]
  U32ToUSize,

  /// Logger Error
  #[snafu(display("Can not create logger"))]
  CannotCreateLogger,

  /// Common Error
  #[snafu(display("Cannot translate given name, name: {}", name))]
  CannotTranslate { name: String },
  #[snafu(display("Actor task has been killed, error: {}", error))]
  ActorTaskBeenKilled { error: tokio::sync::oneshot::error::RecvError },
  #[snafu(display("String parse from bytes error, error: {}", error))]
  StringParseFromBytes { error: std::string::FromUtf8Error },
  #[snafu(display("Future already complete without streaming"))]
  FutureAlreadyCompleted,
}

#[derive(Debug)]
struct HttpRejection {
  message: String,
  code: u16,
}

impl HttpRejection {
  fn new<S: Into<String>>(message: S, code: u16) -> Self {
    Self {
      message: message.into(),
      code,
    }
  }
}

impl warp::reject::Reject for HttpRejection {}

pub enum HttpError {
  CannotGetRedisKeysError,
  CannotMGetRedisError,
}

impl HttpError {
  pub fn reject(&self) -> warp::reject::Rejection {
    match self {
      HttpError::CannotGetRedisKeysError => warp::reject::custom(HttpRejection::new("Cannot get redis keys.", 404)),
      HttpError::CannotMGetRedisError => warp::reject::custom(HttpRejection::new("Cannot mget redis values.", 404)),
    }
  }
}
