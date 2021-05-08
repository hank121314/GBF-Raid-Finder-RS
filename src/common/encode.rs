use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC, PercentEncode};

pub fn percent_encode(src: &str) -> PercentEncode {
  const ENCODER: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

  utf8_percent_encode(src, ENCODER)
}