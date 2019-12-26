use image::Pixel;
use image::RgbImage;
use nalgebra::{clamp, Vector2, Vector3};
use serde::{Deserialize, Deserializer};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Texture {
    texture_path: String,
    width: u32,
    height: u32,
    texture: Option<RgbImage>,
}

impl Texture {
    pub fn load(&mut self, asset_base: &Path) -> Result<(), image::ImageError> {
        let texture = image::open(asset_base.join(self.texture_path.clone()))?.to_rgb();
        self.width = texture.width();
        self.height = texture.height();
        self.texture = Some(texture);

        Ok(())
    }

    pub fn get_color(&self, uv: Vector2<f64>) -> Vector3<f64> {
        assert!(self.texture.is_some());

        let (x, y) = (
            (uv.x * (self.width - 1) as f64) as u32,
            (uv.y * (self.height - 1) as f64) as u32,
        );
        let (x, y) = (clamp(x, 0, self.width), clamp(y, 0, self.height));
        let pixel = self.texture.as_ref().unwrap().get_pixel(x, y);
        let channels = pixel.channels();

        let norm = std::u8::MAX as f64;
        Vector3::new(
            channels[0] as f64 / norm,
            channels[1] as f64 / norm,
            channels[2] as f64 / norm,
        )
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
            width: 0,
            height: 0,
            texture: None,
        })
    }
}
