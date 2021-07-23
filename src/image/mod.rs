use crate::{error, Result};

use crate::proto::raid_boss_raw::RaidBossRaw;
use dssim::{Dssim, DssimImage, ToRGBAPLU};
use imgref::Img;
use load_image::ImageData;

pub struct Comparison {
  origin: RaidBossRaw,
  competitors: Vec<RaidBossRaw>,
  context: Dssim,
}

impl Comparison {
  /// An image comparison service
  ///
  /// Given an origin image and bunch of contestants
  ///
  /// # Specification
  /// 1. Get origin image from the given url.
  /// 2. Get all competitors image from its url.
  /// 3. Use Dssim algorithm to check whether the origin image is same as the competitor image.
  /// 4. If the result of Dssim algorithm is lower than 0.3 (lower is better) it show that competitor is almost same as origin.
  ///
  /// * Most of return values from Dssim algorithm will approach zero, except  Lv120 メドゥーサ and Lvl 120 Medusa.
  ///   The result of  Lvl 120 Medusa will be 0.2xxxxxxx, so I choose 0.3 for insurance purposes.
  ///
  /// # Arguments
  /// * `origin`: origin raid boss that you want to pair with matchers.
  /// * `competitors`: bunch of contestants may match the origin image.
  ///
  /// # Examples
  ///
  /// ```
  /// let origin = RaidBossRaw::with_args(
  ///   "アーカーシャ",
  ///   200,
  ///   r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
  ///   Language::Japanese,
  /// );
  /// let possible_1 = RaidBossRaw::with_args(
  ///   "Akasha",
  ///   200,
  ///   r"https://pbs.twimg.com/media/DumtOgzUYAA_GD3.jpg",
  ///   Language::English,
  /// );
  /// let possible_2 = RaidBossRaw::with_args(
  ///   "Wilnas",
  ///   200,
  ///   r"https://pbs.twimg.com/media/Ed52ry_U0AARvyI.jpg",
  ///   Language::English,
  /// );
  /// let comparison = Comparison::new(origin, vec![possible_1, possible_2]);
  /// let result = comparison.compare().await.unwrap();
  /// assert_eq!("Akasha", result.unwrap().get_boss_name()); // => "Akasha"
  /// ```
  pub fn new<V>(origin: RaidBossRaw, competitors: V) -> Self
  where
    V: IntoIterator<Item = RaidBossRaw>,
  {
    Self {
      origin,
      competitors: competitors.into_iter().collect::<Vec<_>>(),
      context: Dssim::new(),
    }
  }

  pub async fn compare(&self) -> Result<Option<RaidBossRaw>> {
    let origin = self.get_image_from_url(self.origin.get_image()).await?;

    for competitor in self.competitors.clone() {
      let modified = self.get_image_from_url(competitor.get_image()).await?;
      let comparison = self.context.compare(&origin, &modified);
      if comparison.0 < 0.3 {
        return Ok(Some(competitor));
      }
    }

    Ok(None)
  }

  async fn get_image_from_url<S>(&self, url: S) -> Result<DssimImage<f32>>
  where
    S: Into<String>,
  {
    let response = reqwest::get(url.into())
      .await
      .map_err(|error| error::Error::ImageCannotGet { error })?;
    let buffer = response
      .bytes()
      .await
      .map_err(|error| error::Error::BytesParseImage { error })?;

    let img =
      load_image::load_image_data(&buffer, false).map_err(|error| error::Error::ImageParseBytes { error })?;

    self.load_image_to_image_data(img)
  }

  /// Load image from bytes and crop out the bottom 25%
  fn load_image_to_image_data(&self, img: load_image::Image) -> Result<DssimImage<f32>> {
    match img.bitmap {
      ImageData::RGB8(ref bitmap) => {
        self
          .context
          .create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height * 3 / 4))
      }
      ImageData::RGB16(ref bitmap) => {
        self
          .context
          .create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height * 3 / 4))
      }
      ImageData::RGBA8(ref bitmap) => {
        self
          .context
          .create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height * 3 / 4))
      }
      ImageData::RGBA16(ref bitmap) => {
        self
          .context
          .create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height * 3 / 4))
      }
      ImageData::GRAY8(ref bitmap) => {
        self
          .context
          .create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height * 3 / 4))
      }
      ImageData::GRAY16(ref bitmap) => {
        self
          .context
          .create_image(&Img::new(bitmap.to_rgblu(), img.width, img.height * 3 / 4))
      }
      ImageData::GRAYA8(ref bitmap) => {
        self
          .context
          .create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height * 3 / 4))
      }
      ImageData::GRAYA16(ref bitmap) => {
        self
          .context
          .create_image(&Img::new(bitmap.to_rgbaplu(), img.width, img.height * 3 / 4))
      }
    }
    .ok_or(error::Error::ImageToImageData)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{models::Language, proto::raid_boss_raw::RaidBossRaw};

  #[tokio::test]
  async fn translate_akasha_name() {
    let origin = RaidBossRaw::with_args(
      "Lv200 アーカーシャ",
      200,
      r"https://pbs.twimg.com/media/DumtNdnUYAE9PCr.jpg",
      Language::Japanese,
    );
    let possible_1 = RaidBossRaw::with_args(
      "Lvl 200 Wilnas",
      200,
      r"https://pbs.twimg.com/media/Ed52ry_U0AARvyI.jpg",
      Language::English,
    );
    let possible_2 = RaidBossRaw::with_args(
      "Lvl 200 Akasha",
      200,
      r"https://pbs.twimg.com/media/DumtOgzUYAA_GD3.jpg",
      Language::English,
    );
    let comparison = Comparison::new(origin, vec![possible_1, possible_2]);
    let result = comparison.compare().await.unwrap();
    assert_eq!("Lvl 200 Akasha", result.unwrap().get_boss_name()); // => "Lvl 200 Akasha"
  }

  #[tokio::test]
  async fn translate_medusa_hl_name() {
    let origin = RaidBossRaw::with_args(
      "Lv120 メドゥーサ",
      120,
      r"https://pbs.twimg.com/media/CYBki-CUkAQVWW_.jpg",
      Language::Japanese,
    );
    let possible_1 = RaidBossRaw::with_args(
      "Lvl 120 Medusa",
      120,
      r"https://pbs.twimg.com/media/CfqZlIcVIAAp8e_.jpg",
      Language::English,
    );
    let possible_2 = RaidBossRaw::with_args(
      "Lvl 120 Metatron",
      120,
      r"https://pbs.twimg.com/media/DZVlpmXU8AEbF6G.jpg",
      Language::English,
    );
    let comparison = Comparison::new(origin, vec![possible_1, possible_2]);
    let result = comparison.compare().await.unwrap();
    assert_eq!("Lvl 120 Medusa", result.unwrap().get_boss_name()); // => "Lvl 120 Medusa"
  }
}
