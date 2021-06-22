
pub const GRANBLUE_FANTASY_SOURCE: &str = r#"<a href="http://granbluefantasy.jp/" rel="nofollow">グランブルー ファンタジー</a>"#;

pub const SHORTHAND_JAPANESE: &str = "jp";

pub const SHORTHAND_ENGLISH: &str = "en";

pub mod http {
  pub const STREAM_URL: &str = "https://stream.twitter.com/1.1/statuses/filter.json";

  pub const OAUTH_VERSION: &str = "1.0";
}

pub mod redis {
  pub const GBF_PREFIX: &str = "gbf";

  pub const BOSS_KEY_WORD: &str = "boss";

  pub const PERSISTENCE_KEY_WORD: &str = "persistence";

  pub const GBF_TRANSLATOR_KEY: &str = "gbf:translator";

  pub const BOSS_EXPIRE_IN_30_DAYS_TTL: u32 = 2592000;

  pub const TWEET_PERSISTENCE_ONLY_2_HOURS_TTL: u32 = 7200;
}