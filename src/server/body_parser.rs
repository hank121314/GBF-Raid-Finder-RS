use warp::Filter;
use serde::de::DeserializeOwned;

pub fn post_json<T: DeserializeOwned + Send>() -> impl warp::Filter<Extract = (T,), Error = warp::Rejection> + Clone {
  // When accepting a body, we want a JSON body
  // (and to reject huge payloads)...
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}