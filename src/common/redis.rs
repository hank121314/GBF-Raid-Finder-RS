use crate::models::Language;
use crate::proto::raid_boss::RaidBoss;

pub const GBF_PREFIX: &str = "gbf";

pub const PERSISTENCE_KEY_WORD: &str = "persistence";

pub const GBF_TRANSLATOR_KEY: &str = "gbf:translator:map";

pub fn gbf_raid_boss_key(raid_boss: &RaidBoss, lang: Language) -> String {
  let language = match lang {
    Language::English => "en",
    Language::Japanese => "jp",
  };
  format!(
    "{}:{}:{}.{}",
    GBF_PREFIX,
    language,
    raid_boss.get_level(),
    raid_boss.get_boss_name()
  )
}

pub fn gbf_persistence_raid_tweet_key<S: Into<String>>(raid_boss_name: S, lang: Language, tweet_id: u64) -> String {
  let language = match lang {
    Language::English => "en",
    Language::Japanese => "jp",
  };
  format!(
    "{}:{}:{}.{}.{}",
    GBF_PREFIX,
    PERSISTENCE_KEY_WORD,
    language,
    raid_boss_name.into(),
    tweet_id
  )
}

pub fn gbf_get_possible_boss_name(raid_boss: &RaidBoss, lang: Language) -> String {
  let language = match lang {
    Language::English => "en",
    Language::Japanese => "jp",
  };
  format!("{}:{}:{}.*", GBF_PREFIX, language, raid_boss.get_level(),)
}