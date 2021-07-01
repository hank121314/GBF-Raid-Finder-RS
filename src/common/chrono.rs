use std::time::{SystemTime, UNIX_EPOCH};


pub fn current_timestamp() -> String {
  let start = SystemTime::now();
  let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

  since_the_epoch.as_secs().to_string()
}

pub fn current_timestamp_u64() -> u64 {
  let start = SystemTime::now();
  let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

  since_the_epoch.as_secs()
}