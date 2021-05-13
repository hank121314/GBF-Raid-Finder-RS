#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RaidBoss {
  #[prost(string, tag = "1")]
  pub en_name: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub jp_name: ::prost::alloc::string::String,
  #[prost(int32, tag = "3")]
  pub level: i32,
  #[prost(string, tag = "4")]
  pub image: ::prost::alloc::string::String,
}

#[allow(dead_code)]
impl RaidBoss {
  pub fn new() -> RaidBoss {
    ::std::default::Default::default()
  }

  pub fn with_args<S1, S2, S3>(en_name: S1, jp_name: S2, level: i32, image: S3) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
  {
    Self {
      en_name: en_name.into(),
      jp_name: jp_name.into(),
      level,
      image: image.into(),
    }
  }

  pub fn get_en_name(&self) -> &str {
    &self.en_name
  }
  pub fn clear_en_name(&mut self) {
    self.en_name.clear();
  }

  // Param is passed by value, moved
  pub fn set_en_name(&mut self, v: ::std::string::String) {
    self.en_name = v;
  }

  // Mutable pointer to the field.
  // If field is not initialized, it is initialized with default value first.
  pub fn mut_en_name(&mut self) -> &mut ::std::string::String {
    &mut self.en_name
  }

  // Take field
  pub fn take_en_name(&mut self) -> ::std::string::String {
    ::std::mem::replace(&mut self.en_name, ::std::string::String::new())
  }

  pub fn get_jp_name(&self) -> &str {
    &self.jp_name
  }
  pub fn clear_jp_name(&mut self) {
    self.jp_name.clear();
  }

  // Param is passed by value, moved
  pub fn set_jp_name(&mut self, v: ::std::string::String) {
    self.jp_name = v;
  }

  // Mutable pointer to the field.
  // If field is not initialized, it is initialized with default value first.
  pub fn mut_jp_name(&mut self) -> &mut ::std::string::String {
    &mut self.jp_name
  }

  // Take field
  pub fn take_jp_name(&mut self) -> ::std::string::String {
    ::std::mem::replace(&mut self.jp_name, ::std::string::String::new())
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
    ::std::mem::replace(&mut self.image, ::std::string::String::new())
  }
}
