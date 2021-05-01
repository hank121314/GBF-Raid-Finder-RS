use crate::proto::raid_boss::RaidBoss;
use std::time::{SystemTime, UNIX_EPOCH};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC, PercentEncode};
use std::borrow::Borrow;

pub fn percent_encode(src: &str) -> PercentEncode {
  const ENCODER: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

  utf8_percent_encode(src, ENCODER)
}

pub fn current_timestamp() -> String {
  let start = SystemTime::now();
  let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

  since_the_epoch.as_secs().to_string()
}

pub fn gbf_redis_key<T: Borrow<RaidBoss>>(raid_boss: T) -> String {
  format!("gbf:{}.{}", raid_boss.borrow().get_level(), raid_boss.borrow().get_boss_name())
}