use image::RgbImage;
use nalgebra::Vector3;
use serde::{Deserialize, Deserializer};
use std::path::Path;

#[derive(Debug)]
pub struct Texture {
    texture_path: String,
    texture: Option<RgbImage>,
}

impl Texture {
    pub fn load(&mut self, asset_base: &Path) -> Result<(), image::ImageError> {
        self.texture = Some(image::open(asset_base.join(self.texture_path.clone()))?.to_rgb());

        Ok(())
    }

    pub fn get_color(&self, uv: (f64, f64)) -> Vector3<f64> {
        assert!(self.texture.is_some());

        Vector3::new(uv.0, uv.1, 0.0)
    }
}

impl<'de> Deserialize<'de> for Texture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let texture_path: String = Deserialize::deserialize(deserializer)?;

        Ok(Texture {
            texture_path: texture_path.to_string(),
            texture: None,
        })
    }
}
