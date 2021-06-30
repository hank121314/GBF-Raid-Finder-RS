use crate::{
  client::parameter::{Parameter, ParameterConvertible},
  common::{chrono::current_timestamp, encode::percent_encode},
  config::Config,
  error::Error,
  Result,
};
use hmac::{Hmac, Mac, NewMac};
use http::header::{AUTHORIZATION, CONNECTION, CONTENT_TYPE};
use nanoid::nanoid;
use sha1::Sha1;
use std::borrow::Borrow;

/// Container of all oauth headers except signature
pub struct OAuthParameters {
  pub consumer_key: Parameter,
  pub nonce: Parameter,
  pub signature_method: Parameter,
  pub timestamp: Parameter,
  pub token: Parameter,
  pub version: Parameter,
}

impl OAuthParameters {
  pub fn new<S1, S2, S3>(consumer_key: S1, token: S2, version: S3) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
  {
    OAuthParameters {
      consumer_key: ("oauth_consumer_key", consumer_key).into(),
      nonce: ("oauth_nonce", nanoid!()).into(),
      signature_method: ("oauth_signature_method", "HMAC-SHA1").into(),
      timestamp: ("oauth_timestamp", current_timestamp()).into(),
      token: ("oauth_token", token).into(),
      version: ("oauth_version", version).into(),
    }
  }

  /// Convert all params to vec to let outer function generate signature easily.
  pub(super) fn to_vec(&self) -> Vec<Parameter> {
    vec![
      self.consumer_key.clone(),
      self.nonce.clone(),
      self.signature_method.clone(),
      self.timestamp.clone(),
      self.token.clone(),
      self.version.clone(),
    ]
  }
}

pub struct OAuthRequestBuilder {
  pub url: String,
  pub method: String,
  pub config: Config,
  pub oauth: OAuthParameters,
  pub query: Vec<Parameter>,
}

impl OAuthRequestBuilder
{
  pub fn new<S1, S2, I: IntoIterator<Item = Parameter>>(
    url: S1,
    method: S2,
    config: Config,
    oauth: OAuthParameters,
    query: I,
  ) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
  {
    OAuthRequestBuilder {
      url: url.into(),
      method: method.into(),
      config,
      oauth,
      query: query.into_iter().collect::<Vec<_>>(),
    }
  }

  /// Gather all of the parameters included in the request.
  /// There are two such locations for these additional parameters
  /// - the URL (as part of the query string)
  /// - the request body
  /// An HTTP request has parameters that are URL encoded, but you should collect the raw values.
  /// In addition to the request parameters, every oauth_* parameter needs to be included in the signature, so collect those too.
  fn collecting_parameters(&self) -> String {
    let mut percent_encoded_parameters = self
      .oauth
      .to_vec()
      .iter()
      .chain(self.query.iter())
      .map(|parameter| parameter.as_percent_encoding())
      .collect::<Vec<String>>();
    percent_encoded_parameters.sort();

    percent_encoded_parameters.join("&")
  }

  /// To encode the HTTP method, base URL, and parameter string into a single string:
  ///
  /// 1. Convert the HTTP Method to uppercase and set the output string equal to this value.
  /// 2. Append the ‘&’ character to the output string.
  /// 3. Percent encode the URL and append it to the output string.
  /// 4. Append the ‘&’ character to the output string.
  /// 5/ Percent encode the parameter string and append it to the output string.
  fn generate_base_signature_string(&self) -> String {
    let method = self.method.to_uppercase();
    let url = percent_encode(self.url.as_str());
    let parameters = self.collecting_parameters();
    let encoded_parameters = percent_encode(parameters.as_str());

    format!("{}&{}&{}", method, url, encoded_parameters)
  }

  /// The signing key is simply the percent encoded consumer secret,
  /// followed by an ampersand character ‘&’,
  /// followed by the percent encoded token secret.
  fn getting_signing_key(&self) -> String {
    let consumer_secret = percent_encode(self.config.api_secret_key.borrow());
    let oauth_token_secret = percent_encode(self.config.access_token_secret.borrow());

    format!("{}&{}", consumer_secret, oauth_token_secret)
  }

  /// The signature is calculated by passing the signature base string and signing key to the HMAC-SHA1 hashing algorithm.
  /// The output of the HMAC signing function is a binary string. This needs to be base64 encoded to produce the signature string.
  fn create_signature(&self) -> String {
    let base_string = self.generate_base_signature_string();
    let signing_key = self.getting_signing_key();
    let mut digest = Hmac::<Sha1>::new_varkey(signing_key.as_bytes()).expect("Wrong key length");
    digest.update(base_string.as_bytes());

    base64::encode(&digest.finalize().into_bytes())
  }

  /// Return OAuth header field
  fn create_authorization_header(&self) -> String {
    let signature = self.create_signature();
    let oauth_signature: Parameter = ("oauth_signature", signature).into();
    let mut oauth_parameters = self.oauth.to_vec();
    oauth_parameters.push(oauth_signature);
    let header_parameters = oauth_parameters
      .iter()
      .map(|parameter| parameter.as_http_parameter())
      .collect::<Vec<_>>();

    format!("OAuth {}", header_parameters.join(", "))
  }

  /// Return query as url encoded string.
  fn query(&self) -> String {
    self
      .query
      .iter()
      .map(|q| format!("{}={}", q.key, percent_encode(q.value.as_ref())))
      .collect::<Vec<_>>()
      .join("&")
  }

  /// Build RequestBuilder
  ///
  /// If self.method is not a valid Http Method it will return an error.
  pub fn build(&self) -> Result<http::Request<()>> {
    let builder = http::Request::builder();
    let host = format!("{}?{}", self.url.clone(), self.query());

    if let Ok(method) = self.method.clone().parse::<http::Method>() {
      return builder
        .method(method)
        .uri(host)
        .header(CONNECTION, "close")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(AUTHORIZATION, self.create_authorization_header())
        .body(())
        .map_err(|_| Error::CannotBuildRequest);
    }

    Err(Error::InvalidHttpMethod)
  }
}
