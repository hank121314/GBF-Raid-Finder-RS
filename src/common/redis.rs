use crate::{
  client::redis::Redis,
  models::Language,
  proto::{raid_boss::RaidBoss, raid_boss_raw::RaidBossRaw},
  Result,
};
use std::collections::HashMap;

pub const GBF_PREFIX: &str = "gbf";


pub const BOSS_KEY_WORD: &str = "boss";

pub const PERSISTENCE_KEY_WORD: &str = "persistence";

pub const GBF_TRANSLATOR_KEY: &str = "gbf:translator";

pub fn gbf_raid_boss_raw_key(raid_boss_raw: &RaidBossRaw, lang: Language) -> String {
  let language = match lang {
    Language::English => "en",
    Language::Japanese => "jp",
  };
  format!(
    "{}:{}:{}.{}",
    GBF_PREFIX, language, raid_boss_raw.level, raid_boss_raw.boss_name
  )
}

pub fn gbf_raid_boss_keys(level: u32) -> String {
  let level_match = match level {
    0 => "*".to_owned(),
    _ => level.to_string()
  };

  format!("{}:{}:{}.*", GBF_PREFIX, BOSS_KEY_WORD, level_match)
}

pub fn gbf_raid_boss_key(name: &str, raid_boss: &RaidBoss ) -> String {
  format!("{}:{}:{}.{}", GBF_PREFIX, BOSS_KEY_WORD, raid_boss.level, name)
}

pub fn gbf_persistence_raid_tweets_key<S: Into<String>>(raid_boss_name: S) -> String {
  format!(
    "{}:{}:{}.*",
    GBF_PREFIX,
    PERSISTENCE_KEY_WORD,
    raid_boss_name.into(),
  )
}

pub fn gbf_persistence_raid_tweet_key<S: Into<String>>(raid_boss_name: S, tweet_id: u64) -> String {
  format!(
    "{}:{}:{}.{}",
    GBF_PREFIX,
    PERSISTENCE_KEY_WORD,
    raid_boss_name.into(),
    tweet_id
  )
}

pub fn gbf_get_possible_boss_name(raid_boss_raw: &RaidBossRaw, lang: Language) -> String {
  let language = match lang {
    Language::English => "en",
    Language::Japanese => "jp",
  };
  format!("{}:{}:{}.*", GBF_PREFIX, language, raid_boss_raw.level)
}

pub async fn get_translator_map<S: Into<String>>(redis: &Redis, keys: S) -> Result<HashMap<String, String>> {
  let redis_keys = redis.keys(keys).await?;
  let redis_values = redis.mget_string(redis_keys.clone()).await?;

  Ok(
    redis_keys
      .into_iter()
      .enumerate()
      .map(|k| (k.1, redis_values[k.0].clone()))
      .collect(),
  )
}
