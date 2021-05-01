use crate::{models::{Language, Tweet}, proto::raid_boss::RaidBoss};

use lazy_static::lazy_static;
use regex::{Captures, Regex};

lazy_static! {
  static ref JPRAID_REGEX: Regex =
    Regex::new(r"(?P<extra>(?s).*)(?P<battle_id>[0-9A-F]{8}) :参戦ID\n参加者募集！\n(?P<boss>.+)\n(?P<url>.*)")
      .unwrap();
  static ref ENRAID_REGEX: Regex =
    Regex::new(r"(?P<extra>(?s).*)(?P<battle_id>[0-9A-F]{8}) :Battle ID\nI need backup!\n(?P<boss>.+)\n(?P<url>.*)")
      .unwrap();
  static ref BOSS_REGEX: Regex = Regex::new("Lv(?:l )?(?P<level>[0-9]+) (?P<boss_name>.*)").unwrap();
}

pub struct StatusParser {}

impl StatusParser {
  pub fn parse(tweet: Tweet) -> Option<RaidBoss> {
    if let Some(jp_raid) = JPRAID_REGEX.captures(&tweet.text) {
      return Self::match_raid(jp_raid, &tweet, Language::Japanese);
    };
    if let Some(en_raid) = ENRAID_REGEX.captures(&tweet.text) {
      return Self::match_raid(en_raid, &tweet, Language::English);
    }

    None
  }

  fn get_media_image_by_tweet(tweet: &Tweet) -> Option<String> {
    if let Some(medias) = tweet.entities.media.clone() {
      if let Some(media) = medias.first() {
        return Some(media.media_url_https.clone());
      }
    }

    None
  }

  fn match_raid(raid_cap: Captures, tweet: &Tweet, language: Language) -> Option<RaidBoss> {
    if let (Some(boss_cap), Some(image)) = (BOSS_REGEX.captures(&raid_cap["boss"]), Self::get_media_image_by_tweet(&tweet),) {
      let boss_name = boss_cap["boss_name"].to_owned();
      let level = boss_cap["level"].parse::<i32>().unwrap_or(0);
      let mut raid_boss = RaidBoss::new();
      raid_boss.set_boss_name(boss_name);
      raid_boss.set_level(level);
      raid_boss.set_image(image);
      raid_boss.set_language(language.to_string());
      return Some(raid_boss);
    }

    None
  }
}
