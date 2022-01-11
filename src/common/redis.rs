use crate::resources::{
  redis::{BOSS_KEY_WORD, GBF_PREFIX, GBF_TRANSLATOR_KEY, PERSISTENCE_KEY_WORD},
  SHORTHAND_ENGLISH, SHORTHAND_JAPANESE,
};
use crate::{
  client::redis::Redis,
  models::Language,
  proto::{raid_boss::RaidBoss, raid_boss_raw::RaidBossRaw},
  Result,
};
use std::{collections::HashMap, str::FromStr};

/// 
/// Get raw boss redis value with its instance
///
/// # Arguments
///
/// * `raid_boss_raw` - RaidBossRaw instance
///
/// # Example
///
/// ```
/// let raid_boss_raw = RaidBossRaw::apply_args(
///   "Lv200 アーカーシャ",
///   200,
///   r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
///   Language::Japanese,
/// );
/// let key = gbf_raid_boss_raw_key(&raid_boss_raw);
/// assert_eq!("gbf:jp:200.Lv200 アーカーシャ", key);
/// ```
pub fn gbf_raid_boss_raw_key(raid_boss_raw: &RaidBossRaw) -> String {
  let language = match Language::from_str(raid_boss_raw.get_language()).unwrap() {
    Language::Japanese => SHORTHAND_JAPANESE,
    Language::English => SHORTHAND_ENGLISH,
  };

  format!(
    "{}:{}:{}.{}",
    GBF_PREFIX,
    language,
    raid_boss_raw.level,
    raid_boss_raw.get_boss_name()
  )
}

///
/// Get translated boss list with level
/// If level equals to 0 means get all bosses.
///
/// # Arguments
///
/// * `level` - level filter
///
/// # Example
///
/// ```
/// let key = gbf_raid_boss_keys(200);
/// assert_eq!("gbf:boss:200.*", key);
/// ```
pub fn gbf_raid_boss_keys(level: u32) -> String {
  let level_match = match level {
    0 => "*".to_owned(),
    _ => level.to_string(),
  };

  format!("{}:{}:{}.*", GBF_PREFIX, BOSS_KEY_WORD, level_match)
}

/// Get translated boss with its level and language
///
/// # Arguments
///
/// * `lang`: Language we want to take
/// * `raid_boss`: Raid boss information
///
/// # Example:
///
/// ```
/// let raid_boss = RaidBoss::apply_args(
///   "Lvl 200 Akasha",
///   "Lv200 アーカーシャ",
///   200,
///   r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
/// );
/// let jp_key = gbf_raid_boss_key(Language::Japanese, &raid_boss);
/// let en_key = gbf_raid_boss_key(Language::English, &raid_boss);
/// assert_eq!("gbf:boss:200.Lv200 アーカーシャ", jp_key);
/// assert_eq!("gbf:boss:200.Lvl 200 Akasha", en_key);
/// ```
pub fn gbf_raid_boss_key(lang: Language, raid_boss: &RaidBoss) -> String {
  match lang {
    Language::English => {
      format!(
        "{}:{}:{}.{}",
        GBF_PREFIX, BOSS_KEY_WORD, raid_boss.level, raid_boss.en_name
      )
    }
    Language::Japanese => {
      format!(
        "{}:{}:{}.{}",
        GBF_PREFIX, BOSS_KEY_WORD, raid_boss.level, raid_boss.jp_name
      )
    }
  }
}

/// Get raid boss jp name from raid boss raw and translated
///
/// # Arguments
///
/// * `lang`: Language we want to take
/// * `raid_boss_raw`: Raid boss raw information
/// * `translated`: Translated name.
///
/// # Example:
///
/// ```
///let raid_boss_raw = RaidBossRaw::apply_args(
///   "Lv200 アーカーシャ",
///   200,
///   r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
///   Language::Japanese,
/// );
/// let translated = "Lvl 200 Akasha";
/// let jp_key = gbf_raid_boss_jp_key_from_raw(Language::Japanese, &raid_boss_raw, translated);
/// assert_eq!("gbf:boss:200.Lv200 アーカーシャ", jp_key);
/// let raid_boss_raw = RaidBossRaw::apply_args(
///   "Lvl 200 Akasha",
///   200,
///   r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
///   Language::English,
/// );
/// let translated = "Lv200 アーカーシャ";
/// let jp_key = gbf_raid_boss_jp_key_from_raw(Language::English, &raid_boss_raw, translated);
/// assert_eq!("gbf:boss:200.Lv200 アーカーシャ", jp_key);
/// ```
pub fn gbf_raid_boss_jp_key_from_raw(lang: Language, raid_boss_raw: &RaidBossRaw, translated: &str) -> String {
  match lang {
    Language::English => {
      format!(
        "{}:{}:{}.{}",
        GBF_PREFIX, BOSS_KEY_WORD, raid_boss_raw.get_level(), translated,
      )
    }
    Language::Japanese => {
      format!(
        "{}:{}:{}.{}",
        GBF_PREFIX, BOSS_KEY_WORD, raid_boss_raw.get_level(), raid_boss_raw.get_boss_name(),
      )
    }
  }
}

/// Get persistence tweets by raid boss name
///
/// # Arguments
///
/// * `raid_boss_name`: The name which we want to retrieve from persistence
///
/// # Example
///
/// ```
/// let keys = gbf_persistence_raid_tweets_keys("Lv200 アーカーシャ");
/// assert_eq!("gbf:persistence:Lv200 アーカーシャ.*", keys);
/// ```
pub fn gbf_persistence_raid_tweets_keys<S: Into<String>>(raid_boss_name: S) -> String {
  format!("{}:{}:{}.*", GBF_PREFIX, PERSISTENCE_KEY_WORD, raid_boss_name.into())
}

/// Get specific persistence tweet by raid boss name and tweet_id
///
/// # Arguments
///
/// * `raid_boss_name`: The name which we want to retrieve from persistence
/// * `tweet_id`: specific tweet id
///
/// # Example
///
/// ```
/// let key = gbf_persistence_raid_tweet_key("Lv200 アーカーシャ", 1234567890);
/// assert_eq!("gbf:persistence:Lv200 アーカーシャ.1234567890", key);
/// ```
pub fn gbf_persistence_raid_tweet_key<S: Into<String>>(raid_boss_name: S, tweet_id: u64, created: u64) -> String {
  format!(
    "{}:{}:{}.{}.{}",
    GBF_PREFIX,
    PERSISTENCE_KEY_WORD,
    raid_boss_name.into(),
    tweet_id,
    created
  )
}

/// 
/// Get bosses which are at the same level with given raid_boss_raw.
///
/// # Arguments
///
/// * `raid_boss_name`: The raid boss which we want to match its level.
/// * `lang`: The language of keys we want to retrieve.
///
/// # Example
///
/// ```
/// let raid_boss_raw = RaidBossRaw::apply_args(
///   "Lv200 アーカーシャ",
///   200,
///   r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
///   Language::Japanese,
/// );
/// let jp_key = gbf_get_possible_boss_name(&raid_boss_raw, Language::Japanese);
/// let en_key = gbf_get_possible_boss_name(&raid_boss_raw, Language::English);
/// assert_eq!("gbf:jp:200.*", jp_key);
/// assert_eq!("gbf:en:200.*", en_key);
/// ```
pub fn gbf_get_possible_boss_name(raid_boss_raw: &RaidBossRaw, lang: Language) -> String {
  let language = match lang {
    Language::English => SHORTHAND_ENGLISH,
    Language::Japanese => SHORTHAND_JAPANESE,
  };
  format!("{}:{}:{}.*", GBF_PREFIX, language, raid_boss_raw.level)
}

pub async fn get_translator_map(redis: &Redis) -> Result<HashMap<String, String>> {
  let redis_keys = redis.keys(format!("{}:*", GBF_TRANSLATOR_KEY)).await?;
  let redis_values = redis.mget_string(redis_keys.clone()).await?;
  let replace = format!("{}:", GBF_TRANSLATOR_KEY);

  Ok(
    redis_keys
      .into_iter()
      .enumerate()
      .map(|k| (k.1.replace(&replace, ""), redis_values[k.0].clone()))
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_gbf_raid_boss_raw_key() {
    let raid_boss_raw = RaidBossRaw::apply_args(
      "Lv200 アーカーシャ",
      200,
      r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
      Language::Japanese,
    );
    let key = gbf_raid_boss_raw_key(&raid_boss_raw);
    assert_eq!("gbf:jp:200.Lv200 アーカーシャ", key);
  }

  #[test]
  fn test_gbf_raid_boss_keys() {
    let key = gbf_raid_boss_keys(200);
    assert_eq!("gbf:boss:200.*", key);
    let key = gbf_raid_boss_keys(0);
    assert_eq!("gbf:boss:*.*", key);
  }

  #[test]
  fn test_gbf_raid_boss_key() {
    let raid_boss = RaidBoss::apply_args(
      "Lvl 200 Akasha",
      "Lv200 アーカーシャ",
      200,
      r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
    );
    let jp_key = gbf_raid_boss_key(Language::Japanese, &raid_boss);
    let en_key = gbf_raid_boss_key(Language::English, &raid_boss);
    assert_eq!("gbf:boss:200.Lv200 アーカーシャ", jp_key);
    assert_eq!("gbf:boss:200.Lvl 200 Akasha", en_key);
  }

  #[test]
  fn test_gbf_raid_boss_jp_key_from_jp_raw() {
    let raid_boss_raw = RaidBossRaw::apply_args(
      "Lv200 アーカーシャ",
      200,
      r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
      Language::Japanese,
    );
    let translated = "Lvl 200 Akasha";
    let jp_key = gbf_raid_boss_jp_key_from_raw(Language::Japanese, &raid_boss_raw, translated);
    assert_eq!("gbf:boss:200.Lv200 アーカーシャ", jp_key);
    let raid_boss_raw = RaidBossRaw::apply_args(
      "Lvl 200 Akasha",
      200,
      r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
      Language::English,
    );
    let translated = "Lv200 アーカーシャ";
    let jp_key = gbf_raid_boss_jp_key_from_raw(Language::English, &raid_boss_raw, translated);
    assert_eq!("gbf:boss:200.Lv200 アーカーシャ", jp_key);
  }

  #[test]
  fn test_gbf_persistence_raid_tweets_keys() {
    let keys = gbf_persistence_raid_tweets_keys("Lv200 アーカーシャ");
    assert_eq!("gbf:persistence:Lv200 アーカーシャ.*", keys);
  }

  #[test]
  fn test_gbf_persistence_raid_tweet_key() {
    let key = gbf_persistence_raid_tweet_key("Lv200 アーカーシャ", 1234567890, 12345678909999);
    assert_eq!("gbf:persistence:Lv200 アーカーシャ.1234567890.12345678909999", key);
  }

  #[test]
  fn test_gbf_get_possible_boss_name() {
    let raid_boss_raw = RaidBossRaw::apply_args(
      "Lv200 アーカーシャ",
      200,
      r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
      Language::Japanese,
    );
    let jp_key = gbf_get_possible_boss_name(&raid_boss_raw, Language::Japanese);
    let en_key = gbf_get_possible_boss_name(&raid_boss_raw, Language::English);
    assert_eq!("gbf:jp:200.*", jp_key);
    assert_eq!("gbf:en:200.*", en_key);
  }
}
