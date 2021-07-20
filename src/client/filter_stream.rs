use crate::{error, Result};

use futures::{Future, Stream};
use hyper::{client::ResponseFuture, Body, Request};
use log::info;
use serde::de::DeserializeOwned;
use std::{
  pin::Pin,
  task::{Context, Poll},
};

pub struct StreamingSource<T: DeserializeOwned> {
  body: Option<Body>,
  request: Option<Request<Body>>,
  response: Option<ResponseFuture>,
  tweet: Option<T>,
}

impl<T> StreamingSource<T>
where
  T: DeserializeOwned,
{
  pub fn new(request: Request<Body>) -> Self {
    Self {
      request: Some(request),
      response: None,
      body: None,
      tweet: None,
    }
  }
}

impl<T> Unpin for StreamingSource<T> where T: DeserializeOwned {}

impl<T> Stream for StreamingSource<T>
where
  T: DeserializeOwned,
{
  type Item = Result<T>;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    if let Some(req) = self.request.take() {
      let connector = hyper_tls::HttpsConnector::new();
      let client = hyper::Client::builder().build(connector);
      let response = client.request(req);
      self.response = Some(response);
    }

    if let Some(mut response) = self.response.take() {
      match Pin::new(&mut response).poll(cx) {
        Poll::Pending => {
          self.response = Some(response);
          return Poll::Pending;
        }
        Poll::Ready(Err(error)) => return Poll::Ready(Some(Err(error::Error::CannotGetStream { error }))),
        Poll::Ready(Ok(res)) => {
          let status_code = res.status();
          if !status_code.is_success() {
            return Poll::Ready(Some(Err(error::Error::BadResponseError)));
          }

          info!("Connected to twitter streaming api!");
          self.body = Some(res.into_body());
        }
      };
    }

    if let Some(mut body) = self.body.take() {
      return match Pin::new(&mut body).poll_next(cx) {
        Poll::Pending => {
          self.body = Some(body);

          Poll::Pending
        }
        Poll::Ready(None) => Poll::Ready(None),
        Poll::Ready(Some(Err(_))) => {
          self.body = Some(body);

          Poll::Ready(Some(Err(error::Error::StreamEOFError)))
        }
        Poll::Ready(Some(Ok(chunk))) => {
          self.body = Some(body);
          let string =
            String::from_utf8(chunk.to_vec()).map_err(|error| error::Error::StringParseFromBytesError { error })?;
          let data =
            serde_json::from_str::<T>(string.as_ref()).map_err(|error| error::Error::JSONParseError { error })?;
          self.tweet = Some(data);
          if let Some(tweet) = self.tweet.take() {
            return Poll::Ready(Some(Ok(tweet)));
          }

          Poll::Ready(Some(Err(error::Error::StreamEOFError)))
        }
      };
    } else {
      Poll::Ready(Some(Err(error::Error::FutureAlreadyCompleted)))
    }
  }
}
