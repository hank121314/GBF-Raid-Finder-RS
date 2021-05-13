use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Language {
  English,
  Japanese,
}

impl std::fmt::Display for Language {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Language {
  pub fn opposite(&self) -> Self {
    match self {
      Language::English => Language::Japanese,
      Language::Japanese => Language::English,
    }
  }
}

impl std::str::FromStr for Language {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "Japanese" => Ok(Language::Japanese),
      "jp" => Ok(Language::Japanese),
      "English" => Ok(Language::English),
      "en" => Ok(Language::English),
      _ => Err(()),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Media {
  pub media_url_https: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entity {
  pub media: Option<Vec<Media>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
  pub screen_name: String,
  pub profile_image_url_https: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tweet {
  pub id: u64,
  pub text: String,
  pub source: String,
  pub entities: Entity,
  pub timestamp_ms: String,
  pub user: User,
}
