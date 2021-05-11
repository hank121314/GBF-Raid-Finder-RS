use crate::{
  common::redis::{gbf_get_possible_boss_name, GBF_TRANSLATOR_KEY},
  image::Comparison,
  models::Language,
  proto::raid_boss::RaidBoss,
  Redis, Result,
};
use log::info;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// An independent translation task
///
/// # Arguments
/// * `raid_boss` - a RaidBoss that you want to translate.
/// * `redis` - redis client.
/// * `map` - an actor map to memoize translation result.
pub async fn translator_tasks(
  raid_boss: RaidBoss,
  redis: Arc<Redis>,
  map: Arc<RwLock<HashMap<String, String>>>,
) -> Result<()> {
  let boss_name = raid_boss.get_boss_name();
  let to_language = match raid_boss.get_language() {
    "English" => Language::Japanese,
    "Japanese" => Language::English,
    _ => Language::Japanese,
  };
  // Get redis-cli keys for possible_boss
  // ex. gbf:jp:200.*
  let possible_name = gbf_get_possible_boss_name(&raid_boss, to_language);
  let possible_boss_keys = redis.keys(possible_name).await?;
  let possible_bosses = redis.mget_protobuf(possible_boss_keys).await?;

  let comparison = Comparison::new(raid_boss.clone(), possible_bosses);

  match comparison.compare().await? {
    Some(matched) => {
      let translated_name = matched.get_boss_name();
      let mut writable_map = map.write().await;
      writable_map.insert(boss_name.into(), translated_name.into());
      let map_2_redis: HashMap<String, String> = writable_map
        .iter()
        // Pending status empty string should not insert to redis map, cause pending status will be close when application is gone.
        .filter(|k| !k.1.is_empty())
        .map(|k| (k.0.into(), k.1.into()))
        .collect::<HashMap<_, _>>();
      // Drop write lock before writing to redis, reduce redis set time consumption.
      drop(writable_map);
      info!(
        "Translate {} name to {} complete! Writing to redis...",
        boss_name, translated_name
      );
      redis.set_hash_map(GBF_TRANSLATOR_KEY, map_2_redis, 0).await?;

      Ok(())
    }
    None => {
      info!(
        "Cannot translate {}, maybe other language raid_boss is not exist",
        boss_name
      );

      Ok(())
    }
  }
}
