use crate::{
  client::parameters::{OAuthParameters, OAuthRequestBuilder, Parameter},
  config::Config,
  error::Error,
  resources::STREAM_URL,
  Result,
};

use futures::StreamExt;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
/// Internal Representation of a Client
pub struct FilterStreamClient<'a> {
  config: Config<'a>,
  parameters: Vec<Parameter<'a>>,
}

impl<'a> FilterStreamClient<'a> {
  pub fn new(config: Config<'a>, track: Vec<&'a str>, stall_warning: &'a str) -> Self {
    let stall_warning: Parameter = ("stall_warning", stall_warning).into();
    let track: Parameter = ("track", track.join(",")).into();

    FilterStreamClient {
      config,
      parameters: vec![stall_warning, track],
    }
  }

  pub async fn stream<T: DeserializeOwned + Send>(&self, sender: Sender<T>) -> Result<()> {
    let oauth = OAuthParameters::new(self.config.api_key.clone(), self.config.access_token.clone(), "1.0");
    let oauth_builder = OAuthRequestBuilder::new(
      STREAM_URL,
      reqwest::Method::POST.as_str(),
      self.config.clone(),
      oauth,
      self.parameters.as_slice(),
    );

    if let Some(request) = oauth_builder.build() {
      let mut response = request
        .send()
        .await
        .map_err(|error| Error::CannotGetStream { error })?
        .bytes_stream();

      while let Some(item) = response.next().await {
        let bytes = item.map_err(|error| Error::BytesParseError { error })?;
        let string = String::from_utf8(bytes.to_vec()).unwrap();
        let data = serde_json::from_str::<T>(&string).map_err(|error| Error::JSONParseError { error })?;
        sender.send(data).await.map_err(|_| Error::SenderSendError)?;
      }

      return Ok(());
    }

    Err(Error::CannotBuildRequest)
  }
}
