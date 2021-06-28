use crate::{
  common::redis::{gbf_get_possible_boss_name, gbf_raid_boss_key},
  image::Comparison,
  models::Language,
  proto::{raid_boss::RaidBoss, raid_boss_raw::RaidBossRaw},
  resources::redis::{BOSS_EXPIRE_IN_30_DAYS_TTL, GBF_TRANSLATOR_KEY},
  Redis, Result,
};
use log::{info, error};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::RwLock;

/// An independent translation task
///
/// # Specification
/// 1. Get all possible boss names (a possible boss means it level is same as the given boss).
/// 2. Remove the boss which is already translated from possible bosses.
/// 3. Mget all possible boss.
/// 4. Use `image::Comparison` to get the correspond boss.
/// - If result from 4. is None(translated boss not found), return emtpy string.
/// 5. Get translated name, it will be empty string or a real value.
/// 6. If translated name is not empty set the boss in redis.
///
/// # Arguments
/// * `raid_boss` - a RaidBoss that you want to translate.
/// * `redis` - a redis client.
/// * `map` - an actor map to memoize translation result.
pub async fn translator_tasks(
  raid_boss_raw: RaidBossRaw,
  redis: Arc<Redis>,
  map: Arc<RwLock<HashMap<String, String>>>,
) -> Result<()> {
  let boss_name = raid_boss_raw.get_boss_name();
  let from_language = Language::from_str(raid_boss_raw.get_language()).unwrap();
  let to_language = from_language.opposite();

  // Get current translation map
  let readable_map = map.read().await;
  // Get the name map which is already translated.
  let paired_keys = readable_map
    .clone()
    .into_iter()
    .filter(|s| !s.1.is_empty())
    .map(|s| (s.0))
    .collect::<Vec<_>>();
  drop(readable_map);

  // Get redis-cli keys for possible_boss
  // ex. gbf:jp:200.*
  let possible_name = gbf_get_possible_boss_name(&raid_boss_raw, to_language);

  // filter out the possible_name which is already translated.
  let possible_boss_keys = redis
    .keys(possible_name)
    .await?
    .into_iter()
    .filter(|possible_key| match possible_key.split(".").last() {
      Some(last) => !paired_keys.iter().any(|key| key == last),
      None => false,
    })
    .collect::<Vec<_>>();

  let possible_bosses = redis.mget_protobuf(possible_boss_keys).await?;

  let comparison = Comparison::new(raid_boss_raw.clone(), possible_bosses);

  let translated_name: String = match comparison.compare().await? {
    Some(matched) => {
      let translated_name = matched.get_boss_name();
      let mut writable_map = map.write().await;
      writable_map.insert(boss_name.into(), translated_name.into());
      writable_map.insert(translated_name.into(), boss_name.into());
      // Drop write lock before writing to redis, it will prevent map from getting lock during redis setting operation.
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
      error!(
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
    // The first argument of RaidBoss::with_args will always be en_name, if from_language is english en_name should be its name.
    names = (raid_boss_raw.get_boss_name(), translated_name.as_str());
  }

  let raid_boss = RaidBoss::with_args(names.0, names.1, raid_boss_raw.get_level(), raid_boss_raw.get_image());

  // Raid Finder always chose japanese name as redis key
  let redis_key_name = raid_boss.get_jp_name();

  match redis_key_name.is_empty() {
    true => Ok(()),
    false => {
      redis
        .set_protobuf(
          gbf_raid_boss_key(Language::Japanese, &raid_boss),
          raid_boss,
          BOSS_EXPIRE_IN_30_DAYS_TTL,
        )
        .await
    }
  }
}
