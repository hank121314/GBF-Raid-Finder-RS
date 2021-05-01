use crate::{
  common::{current_timestamp, percent_encode},
  config::Config,
};
use hmac::{Hmac, Mac, NewMac};
use nanoid::nanoid;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONNECTION, CONTENT_TYPE};
use sha1::Sha1;
use std::borrow::{Borrow, Cow};

/// Trait for Twitter OAuth 1.0a formation.
///
/// For Twitter OAuth 1.0a, parameters will have two kinds of representation.
/// The first one is used to create signature or query. It should format as `format("{}={}")`.
/// The second one is used to create "Oauth Authorization Header". It should format as `format("{}=\"{}\"")`.
pub trait ParameterConvertible {
  fn as_percent_encoding(&self) -> String;
  fn as_http_parameter(&self) -> String;
  fn as_http_query(&self) -> (&str, &str);
}

#[derive(Clone, Debug)]
pub struct Parameter<'a> {
  pub key: Cow<'a, str>,
  pub value: Cow<'a, str>,
}

impl<S1, S2> From<(S1, S2)> for Parameter<'_>
where
  S1: Into<String>,
  S2: Into<String>,
{
  fn from((key, value): (S1, S2)) -> Self {
    Parameter {
      key: Cow::Owned(key.into()),
      value: Cow::Owned(value.into()),
    }
  }
}

impl ParameterConvertible for Parameter<'_> {
  fn as_percent_encoding(&self) -> String {
    format!(
      "{}={}",
      percent_encode(self.key.borrow()),
      percent_encode(self.value.borrow())
    )
  }

  fn as_http_parameter(&self) -> String {
    format!(
      "{}=\"{}\"",
      percent_encode(self.key.borrow()),
      percent_encode(self.value.borrow())
    )
  }

  /// This type is for reqwest query, it will takes a slice of &str tuples.
  fn as_http_query(&self) -> (&str, &str) {
    (self.key.borrow(), self.value.borrow())
  }
}

/// Container of all oauth headers except signature
pub struct OAuthParameters<'a> {
  pub consumer_key: Parameter<'a>,
  pub nonce: Parameter<'a>,
  pub signature_method: Parameter<'a>,
  pub timestamp: Parameter<'a>,
  pub token: Parameter<'a>,
  pub version: Parameter<'a>,
}

impl OAuthParameters<'_> {
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

pub struct OAuthRequestBuilder<'a> {
  pub url: String,
  pub method: String,
  pub config: Config<'a>,
  pub oauth: OAuthParameters<'a>,
  pub query: Vec<Parameter<'a>>,
}

impl<'a> OAuthRequestBuilder<'a> {
  pub fn new<S1, S2, V>(url: S1, method: S2, config: Config<'a>, oauth: OAuthParameters<'a>, query: V) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
    V: Into<Vec<Parameter<'a>>>,
  {
    OAuthRequestBuilder {
      url: url.into(),
      method: method.into(),
      config,
      oauth,
      query: query.into(),
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

  /// Return query as reqwest acceptable type
  fn query(&self) -> Vec<(&str, &str)> {
    self
      .query
      .iter()
      .map(|q| q.as_http_query())
      .collect::<Vec<(&str, &str)>>()
  }

  /// Build RequestBuilder
  ///
  /// If self.method is not a valid Http Method it will return null.
  pub fn build(&self) -> Option<reqwest::RequestBuilder> {
    let client = reqwest::Client::new();

    let url = self.url.clone();
    let mut headers = HeaderMap::new();
    let query = self.query();

    headers.insert(CONNECTION, "close".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/x-www-form-urlencoded".parse().unwrap());
    headers.insert(AUTHORIZATION, self.create_authorization_header().parse().unwrap());

    if let Ok(method) = self.method.clone().parse::<reqwest::Method>() {
      return Some(client.request(method, url).headers(headers).query(&query));
    }

    None
  }
}
