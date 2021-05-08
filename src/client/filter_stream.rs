use crate::{error, Result};

use futures::{AsyncRead, Future, Stream};
use http::Request;
use isahc::{AsyncBody, RequestExt, ResponseFuture};
use serde::de::DeserializeOwned;
use std::{
  pin::Pin,
  task::{Context, Poll},
};

pub struct StreamingSource<T: DeserializeOwned + std::fmt::Debug> {
  body: Option<AsyncBody>,
  request: Option<Request<()>>,
  response: Option<ResponseFuture<'static>>,
  tweet: Option<T>,
}

impl<T> StreamingSource<T>
where
  T: DeserializeOwned + std::fmt::Debug,
{
  pub fn new(request: Request<()>) -> Self {
    Self {
      request: Some(request),
      response: None,
      body: None,
      tweet: None,
    }
  }
}

impl<T> Unpin for StreamingSource<T> where T: DeserializeOwned + std::fmt::Debug {}

impl<T> Stream for StreamingSource<T>
where
  T: DeserializeOwned + std::fmt::Debug,
{
  type Item = Result<T>;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    if let Some(req) = self.request.take() {
      self.response = Some(req.send_async());
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

          self.body = Some(res.into_body());
        }
      };
    }

    if let Some(mut body) = self.body.take() {
      let mut buffer = [0; 16384];
      loop {
        return match Pin::new(&mut body).poll_read(cx, &mut buffer) {
          Poll::Pending => {
            self.body = Some(body);
            Poll::Pending
          }
          Poll::Ready(Err(_)) => {
            self.body = Some(body);
            Poll::Ready(Some(Err(error::Error::StreamEOFError)))
          }
          Poll::Ready(Ok(len)) => {
            let string = String::from_utf8(buffer[..len].to_owned())
              .map_err(|error| error::Error::StringParseFromBytesError { error })?;
            self.body = Some(body);
            let data =
              serde_json::from_str::<T>(string.as_ref()).map_err(|error| error::Error::JSONParseError { error })?;
            self.tweet = Some(data);
            if let Some(tweet) = self.tweet.take() {
              return Poll::Ready(Some(Ok(tweet)));
            }
            Poll::Ready(Some(Err(error::Error::StreamEOFError)))
          }
        };
      }
    } else {
      return Poll::Ready(Some(Err(error::Error::FutureAlreadyCompleted)));
    }
  }
}
