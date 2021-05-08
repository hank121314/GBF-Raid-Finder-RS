use crate::models::Language;
use crate::proto::raid_boss::RaidBoss;
use std::borrow::Borrow;

pub fn gbf_translator_key() -> String {
  "gbf:translator:map".into()
}

pub fn gbf_raid_boss_key<T: Borrow<RaidBoss>>(raid_boss: T, lang: Language) -> String {
  let language = match lang {
    Language::English => "en",
    Language::Japanese => "jp"
  };
  format!(
    "gbf:{}:{}.{}",
    language,
    raid_boss.borrow().get_level(),
    raid_boss.borrow().get_boss_name()
  )
}

pub fn gbf_get_possible_boss_name<T: Borrow<RaidBoss>>(raid_boss: T, lang: Language) -> String {
  let language = match lang {
    Language::English => "en",
    Language::Japanese => "jp"
  };
  format!("gbf:{}:{}.*", language, raid_boss.borrow().get_level(),)
}

pub fn gbf_raid_boss_keys<T: Borrow<RaidBoss>>(raid_boss: T) -> String {
  format!("gbf:{}", raid_boss.borrow().get_level())
}
