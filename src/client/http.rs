use crate::{
  client::{
    filter_stream::StreamingSource,
    oauth::{OAuthParameters, OAuthRequestBuilder},
    parameter::Parameter,
  },
  config::Config,
  resources::http::OAUTH_VERSION,
  Result,
};
use serde::de::DeserializeOwned;

#[derive(Clone)]
/// Internal Representation of a Client
pub struct FilterStreamClient {
  config: Config,
  parameters: Vec<Parameter>,
}

impl FilterStreamClient {
  /// Create an Twitter Filter Stream Client with `Config`, `track` and `stall_warning`
  ///
  /// # Arguments
  /// * `config` - an `Config` instance which contain your environment variable
  /// * `track` - an array of string you want to track
  /// * `stall_warning` - enable stall_warning or not
  ///
  /// # Examples
  ///
  /// ```
  /// let config = Config::new()?;
  /// let client = FilterStreamClient::new(config, vec!["twitter", "stream"], "true");
  /// ```
  pub fn new<I: IntoIterator<Item = S>, S>(config: Config, track: I, stall_warning: S) -> Self
  where
    S: Into<String>,
  {
    let stall_warning: Parameter = ("stall_warning", stall_warning.into()).into();
    // Convert track array into parameter, ex. `(track, twitter, stream)`
    let track: Parameter = (
      "track",
      track.into_iter().map(|s| s.into()).collect::<Vec<_>>().join(","),
    )
      .into();

    FilterStreamClient {
      config,
      parameters: vec![stall_warning, track],
    }
  }

  pub async fn oauth_stream<S, T: DeserializeOwned>(&self, url: S) -> Result<StreamingSource<T>>
  where
    S: Into<String>,
  {
    let oauth = OAuthParameters::new(self.config.api_key.clone(), self.config.access_token.clone(), OAUTH_VERSION);
    let oauth_builder = OAuthRequestBuilder::new(
      url,
      reqwest::Method::POST.as_str(),
      self.config.clone(),
      oauth,
      self.parameters.clone(),
    );
    let request = oauth_builder.build()?;

    Ok(StreamingSource::new(request))
  }
}
