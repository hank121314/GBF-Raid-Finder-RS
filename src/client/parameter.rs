use crate::common::encode::percent_encode;

use std::borrow::Borrow;

/// Trait for Twitter OAuth 1.0a formation.
///
/// For Twitter OAuth 1.0a, parameters will have two kinds of representation.
/// The first one is used to create signature or query. It should format as `format("{}={}")`.
/// The second one is used to create "Oauth Authorization Header". It should format as `format("{}=\"{}\"")`.
pub trait ParameterConvertible {
  fn as_percent_encoding(&self) -> String;
  fn as_http_parameter(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct Parameter {
  pub key: String,
  pub value: String,
}

impl<S1, S2> From<(S1, S2)> for Parameter
where
  S1: Into<String>,
  S2: Into<String>,
{
  fn from((key, value): (S1, S2)) -> Self {
    Parameter {
      key: key.into(),
      value: value.into(),
    }
  }
}

impl ParameterConvertible for Parameter {
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
}
