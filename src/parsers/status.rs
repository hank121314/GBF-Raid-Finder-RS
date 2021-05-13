use crate::{
  models::{Language, Tweet},
  proto::{raid_boss_raw::RaidBossRaw, raid_tweet::RaidTweet},
};

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
  pub fn parse(tweet: Tweet) -> Option<(RaidBossRaw, RaidTweet)> {
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

  fn match_raid(raid_cap: Captures, tweet: &Tweet, language: Language) -> Option<(RaidBossRaw, RaidTweet)> {
    let boss_name = raid_cap["boss"].to_owned();
    let mut level = 0;
    let mut raid_boss = RaidBossRaw::new();
    if let Some(boss_cap) = BOSS_REGEX.captures(&raid_cap["boss"]) {
      level = boss_cap["level"].parse::<i32>().unwrap_or(0);
    }
    raid_boss.set_boss_name(boss_name.clone());
    raid_boss.set_level(level);
    raid_boss.set_language(language.to_string());

    match Self::get_media_image_by_tweet(&tweet) {
      Some(image) => {
        raid_boss.set_image(image);
        let created = tweet.timestamp_ms.parse::<u64>().unwrap();
        let raid_tweet = RaidTweet::with_args(
          tweet.id,
          &tweet.user.screen_name,
          created,
          boss_name,
          &raid_cap["battle_id"],
          &raid_cap["extra"],
          language,
          &tweet.user.profile_image_url_https,
        );

        Some((raid_boss, raid_tweet))
      }
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::{Entity, Language, Media, Tweet, User};

  #[test]
  fn test_jp_parser() {
    let tweet = Tweet {
      id: 1390247452125458434,
      text: "麻痹延长 7D705AE2 :参戦ID\n参加者募集！\nLv150 プロトバハムート\nhttps://t.co/MYfvDDTSrh".into(),
      source: r#"<a href="http://granbluefantasy.jp/" rel="nofollow">グランブルー ファンタジー</a>"#.into(),
      entities: Entity {
        media: Some(vec![Media {
          media_url_https: "https://pbs.twimg.com/media/CdL4WyxUYAIXPb8.jpg".into(),
        }]),
      },
      timestamp_ms: "1620698515453".to_string(),
      user: User {
        screen_name: "".to_string(),
        profile_image_url_https: "".to_string(),
      },
    };
    let raid_boss = StatusParser::parse(tweet).unwrap().0;
    assert_eq!("Lv150 プロトバハムート", raid_boss.get_boss_name());
    assert_eq!(150, raid_boss.get_level());
    assert_eq!("https://pbs.twimg.com/media/CdL4WyxUYAIXPb8.jpg", raid_boss.get_image());
    assert_eq!(Language::Japanese.to_string(), raid_boss.get_language());
  }
}
