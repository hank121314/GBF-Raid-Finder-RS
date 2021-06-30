use crate::models::Language;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RaidBossRaw {
  #[prost(string, tag = "1")]
  pub boss_name: ::prost::alloc::string::String,
  #[prost(int32, tag = "2")]
  pub level: i32,
  #[prost(string, tag = "3")]
  pub image: ::prost::alloc::string::String,
  #[prost(string, tag = "4")]
  pub language: ::prost::alloc::string::String,
}

#[allow(dead_code)]
impl RaidBossRaw {
  pub fn new() -> RaidBossRaw {
    ::std::default::Default::default()
  }

  pub fn with_args<S1, S2>(boss_name: S1, level: i32, image: S2, language: Language) -> Self where S1: Into<String>, S2: Into<String> {
    Self {
      boss_name: boss_name.into(),
      level,
      image: image.into(),
      language: language.to_string(),
    }
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
    ::std::mem::take(&mut self.boss_name)
  }

  pub fn get_level(&self) -> i32 {
    self.level
  }
  pub fn clear_level(&mut self) {
    self.level = 0;
  }

  // Param is passed by value, moved
  pub fn set_level(&mut self, v: i32) {
    self.level = v;
  }

  pub fn get_image(&self) -> &str {
    &self.image
  }
  pub fn clear_image(&mut self) {
    self.image.clear();
  }

  // Param is passed by value, moved
  pub fn set_image(&mut self, v: ::std::string::String) {
    self.image = v;
  }

  // Mutable pointer to the field.
  // If field is not initialized, it is initialized with default value first.
  pub fn mut_image(&mut self) -> &mut ::std::string::String {
    &mut self.image
  }

  // Take field
  pub fn take_image(&mut self) -> ::std::string::String {
    ::std::mem::take(&mut self.image)
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
    ::std::mem::take(&mut self.language)
  }
}