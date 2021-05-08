use crate::{Result, error::Error};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::{
  policy::compound::{
    roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
  },
  RollingFileAppender,
};
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::path::Path;

const SIZE_LIMIT: u64 = 1024 * 1024;
const WINDOW_SIZE: u32 = 7;

pub fn create_logger<S1, S2>(log_path: S1, file_name: S2, log_level: i8) -> Result<()>
  where
    S1: Into<String>,
    S2: Into<String>,
{
  let log_level = match log_level {
    n if n > 4 => LevelFilter::Error,
    4 => LevelFilter::Warn,
    3 => LevelFilter::Info,
    2 => LevelFilter::Debug,
    1 => LevelFilter::Trace,
    _ => LevelFilter::Trace,
  };
  let file_name_str: &str = &file_name.into();
  let log_path_str: &str = &log_path.into();
  // Fixed window roller pattern
  let pattern = format!("{}.{{}}", file_name_str);
  let fixed_window_roller = Box::new(
    FixedWindowRoller::builder()
      .build(pattern.as_str(), WINDOW_SIZE)
      .unwrap(),
  );

  let size_trigger = Box::new(SizeTrigger::new(SIZE_LIMIT));

  let encoder =
    PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} [{h({l})}] {m} ((at: {M}((line: {L})))){n}");

  let path = Path::new(log_path_str).join(file_name_str);

  if let Ok(file) = RollingFileAppender::builder()
    .encoder(Box::new(encoder.clone()))
    .build(
      path,
      Box::new(CompoundPolicy::new(size_trigger, fixed_window_roller)),
    )
  {
    if let Ok(config) = log4rs::Config::builder()
      .appender(Appender::builder().build("log_file", Box::new(file)))
      .build(Root::builder().appender("log_file").build(log_level))
    {
      if log4rs::init_config(config).is_ok() {
        return Ok(());
      }
    }
  } else {
    let std = ConsoleAppender::builder()
      .encoder(Box::new(encoder))
      .build();
    if let Ok(config) = log4rs::Config::builder()
      .appender(Appender::builder().build("stdout", Box::new(std)))
      .build(Root::builder().appender("stdout").build(log_level))
    {
      if log4rs::init_config(config).is_ok() {
        return Ok(());
      }
    }
  }

  Err(Error::CannotCreateLogger)
}