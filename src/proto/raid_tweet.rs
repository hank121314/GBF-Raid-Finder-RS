use crate::{error, models::Language, Result};
use ::prost::Message;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RaidTweet {
  #[prost(uint64, tag = "1")]
  pub tweet_id: u64,
  #[prost(string, tag = "2")]
  pub screen_name: ::prost::alloc::string::String,
  #[prost(string, tag = "3")]
  pub boss_name: ::prost::alloc::string::String,
  #[prost(string, tag = "4")]
  pub raid_id: ::prost::alloc::string::String,
  #[prost(string, tag = "5")]
  pub text: ::prost::alloc::string::String,
  #[prost(uint64, tag = "6")]
  pub created: u64,
  #[prost(string, tag = "7")]
  pub language: ::prost::alloc::string::String,
  #[prost(string, tag = "8")]
  pub profile_image: ::prost::alloc::string::String,
}

#[allow(dead_code)]
impl RaidTweet {
  pub fn new() -> RaidTweet {
    ::std::default::Default::default()
  }

  pub fn with_args<S1, S2, S3, S4, S5>(
    tweet_id: u64,
    screen_name: S1,
    created: u64,
    boss_name: S2,
    raid_id: S3,
    text: S4,
    language: Language,
    profile_image: S5,
  ) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
    S4: Into<String>,
    S5: Into<String>,
  {
    Self {
      tweet_id,
      screen_name: screen_name.into(),
      created,
      boss_name: boss_name.into(),
      raid_id: raid_id.into(),
      text: text.into(),
      language: language.to_string(),
      profile_image: profile_image.into(),
    }
  }

  pub fn get_tweet_id(&self) -> u64 {
    self.tweet_id
  }
  pub fn clear_tweet_id(&mut self) {
    self.tweet_id = 0;
  }

  // Param is passed by value, moved
  pub fn set_tweet_id(&mut self, v: u64) {
    self.tweet_id = v;
  }

  pub fn get_screen_name(&self) -> &str {
    &self.screen_name
  }
  pub fn clear_screen_name(&mut self) {
    self.screen_name.clear();
  }

  // Param is passed by value, moved
  pub fn set_screen_name(&mut self, v: ::std::string::String) {
    self.screen_name = v;
  }

  // Mutable pointer to the field.
  // If field is not initialized, it is initialized with default value first.
  pub fn mut_screen_name(&mut self) -> &mut ::std::string::String {
    &mut self.screen_name
  }

  // Take field
  pub fn take_screen_name(&mut self) -> ::std::string::String {
    ::std::mem::replace(&mut self.screen_name, ::std::string::String::new())
  }

  pub fn get_boss_name(&self) -> &str {
    &self.boss_name
  }
  pub fn clear_boss_name(&mut self) {
    self.boss_name.clear();
  }

  // Param is passed by value, moved
  pub fn set_boss_name(&mut self, v: ::std::string::String) {
    self.boss_name = v;
  }

  // Mutable pointer to the field.
  // If field is not initialized, it is initialized with default value first.
  pub fn mut_boss_name(&mut self) -> &mut ::std::string::String {
    &mut self.boss_name
  }

  // Take field
  pub fn take_boss_name(&mut self) -> ::std::string::String {
    ::std::mem::replace(&mut self.boss_name, ::std::string::String::new())
  }

  pub fn get_raid_id(&self) -> &str {
    &self.raid_id
  }
  pub fn clear_raid_id(&mut self) {
    self.raid_id.clear();
  }

  // Param is passed by value, moved
  pub fn set_raid_id(&mut self, v: ::std::string::String) {
    self.raid_id = v;
  }

  // Mutable pointer to the field.
  // If field is not initialized, it is initialized with default value first.
  pub fn mut_raid_id(&mut self) -> &mut ::std::string::String {
    &mut self.raid_id
  }

  // Take field
  pub fn take_raid_id(&mut self) -> ::std::string::String {
    ::std::mem::replace(&mut self.raid_id, ::std::string::String::new())
  }

  pub fn get_text(&self) -> &str {
    &self.text
  }
  pub fn clear_text(&mut self) {
    self.text.clear();
  }

  // Param is passed by value, moved
  pub fn set_text(&mut self, v: ::std::string::String) {
    self.text = v;
  }

  // Mutable pointer to the field.
  // If field is not initialized, it is initialized with default value first.
  pub fn mut_text(&mut self) -> &mut ::std::string::String {
    &mut self.text
  }

  // Take field
  pub fn take_text(&mut self) -> ::std::string::String {
    ::std::mem::replace(&mut self.text, ::std::string::String::new())
  }

  pub fn get_created(&self) -> u64 {
    self.created
  }
  pub fn clear_created(&mut self) {
    self.created = 0;
  }

  // Param is passed by value, moved
  pub fn set_created(&mut self, v: u64) {
    self.created = v;
  }

  pub fn get_language(&self) -> &str {
    &self.language
  }
  pub fn clear_language(&mut self) {
    self.language.clear();
  }

  // Param is passed by value, moved
  pub fn set_language(&mut self, v: ::std::string::String) {
    self.language = v;
  }

  // Mutable pointer to the field.
  // If field is not initialized, it is initialized with default value first.
  pub fn mut_language(&mut self) -> &mut ::std::string::String {
    &mut self.language
  }

  // Take field
  pub fn take_language(&mut self) -> ::std::string::String {
    ::std::mem::replace(&mut self.language, ::std::string::String::new())
  }

  pub fn get_profile_image(&self) -> &str {
    &self.profile_image
  }
  pub fn clear_profile_image(&mut self) {
    self.profile_image.clear();
  }

  // Param is passed by value, moved
  pub fn set_profile_image(&mut self, v: ::std::string::String) {
    self.profile_image = v;
  }

  // Mutable pointer to the field.
  // If field is not initialized, it is initialized with default value first.
  pub fn mut_profile_image(&mut self) -> &mut ::std::string::String {
    &mut self.profile_image
  }

  // Take field
  pub fn take_profile_image(&mut self) -> ::std::string::String {
    ::std::mem::replace(&mut self.profile_image, ::std::string::String::new())
  }

  pub fn to_bytes(&self) -> Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    self
      .encode(&mut bytes)
      .map_err(|error| error::Error::ProtobufWriteError { error })?;

    Ok(bytes)
  }
}
