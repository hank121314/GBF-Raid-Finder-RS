use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Language {
  English,
  Japanese,
}

impl std::fmt::Display for Language {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Media {
  pub media_url_https: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Entity {
  pub media: Option<Vec<Media>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tweet {
  pub id: u64,
  pub text: String,
  pub source: String,
  pub entities: Entity,
}
