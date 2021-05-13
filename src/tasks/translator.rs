use crate::common::redis::{gbf_raid_boss_key, GBF_TRANSLATOR_KEY};
use crate::{
  common::redis::gbf_get_possible_boss_name,
  image::Comparison,
  models::Language,
  proto::{raid_boss::RaidBoss, raid_boss_raw::RaidBossRaw},
  resources::BOSS_EXPIRE_IN_30_DAYS_TTL,
  Redis, Result,
};
use log::info;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::RwLock;

/// An independent translation task
///
/// # Arguments
/// * `raid_boss` - a RaidBoss that you want to translate.
/// * `redis` - redis client.
/// * `map` - an actor map to memoize translation result.
pub async fn translator_tasks(
  raid_boss_raw: RaidBossRaw,
  redis: Arc<Redis>,
  map: Arc<RwLock<HashMap<String, String>>>,
) -> Result<()> {
  let boss_name = raid_boss_raw.get_boss_name();
  let from_language = Language::from_str(raid_boss_raw.get_language()).unwrap();
  let to_language = from_language.opposite();
  // Get redis-cli keys for possible_boss
  // ex. gbf:jp:200.*
  let possible_name = gbf_get_possible_boss_name(&raid_boss_raw, to_language);
  let possible_boss_keys = redis.keys(possible_name).await?;
  let possible_bosses = redis.mget_protobuf(possible_boss_keys).await?;

  let comparison = Comparison::new(raid_boss_raw.clone(), possible_bosses);

  let translated_name: String = match comparison.compare().await? {
    Some(matched) => {
      let translated_name = matched.get_boss_name();
      let mut writable_map = map.write().await;
      writable_map.insert(boss_name.into(), translated_name.into());
      writable_map.insert(translated_name.into(), boss_name.into());
      // Drop write lock before writing to redis, reduce redis set time consumption.
      drop(writable_map);
      let map_2_redis = vec![
        (format!("{}:{}", GBF_TRANSLATOR_KEY, boss_name), translated_name),
        (format!("{}:{}", GBF_TRANSLATOR_KEY, translated_name), boss_name),
      ];
      info!(
        "Translate {} name to {} complete! Writing to redis...",
        boss_name, translated_name
      );
      redis.set_multiple_string(map_2_redis).await?;
      Some(translated_name.into())
    }
    None => {
      info!(
        "Cannot translate {}, maybe other language raid_boss is not exist",
        boss_name
      );
      let mut writable_map = map.write().await;
      writable_map.remove(boss_name);

      None
    }
  }
  .unwrap_or_else(|| "".into());

  let mut names: (&str, &str) = (translated_name.as_str(), raid_boss_raw.get_boss_name());
  if from_language == Language::English {
    names = (raid_boss_raw.get_boss_name(), translated_name.as_str());
  }

  let raid_boss = RaidBoss::with_args(names.0, names.1, raid_boss_raw.get_level(), raid_boss_raw.get_image());

  let redis_key_name = raid_boss.get_jp_name();

  if redis_key_name.is_empty() {
    return Ok(());
  }

  redis.set_protobuf(gbf_raid_boss_key(redis_key_name, &raid_boss), raid_boss, BOSS_EXPIRE_IN_30_DAYS_TTL).await
}
