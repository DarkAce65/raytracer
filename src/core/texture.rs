use image::Pixel;
use image::RgbImage;
use nalgebra::{clamp, Vector2, Vector3};
use std::fmt;
use std::path::Path;

#[derive(Clone)]
pub struct Texture {
    texture_path: String,
    width: u32,
    height: u32,
    texture: Option<RgbImage>,
}

impl fmt::Debug for Texture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Texture {{ width: {}, height: {}, texture_path: {} }}",
            self.width, self.height, self.texture_path
        )
    }
}

impl Texture {
    pub fn new(texture_path: &str) -> Self {
        Self {
            texture_path: texture_path.to_string(),
            width: 0,
            height: 0,
            texture: None,
        }
    }

    pub fn load(&mut self, asset_base: &Path) -> Result<(), image::ImageError> {
        assert!(self.texture.is_none());

        let texture = image::open(asset_base.join(self.texture_path.clone()))?.to_rgb8();
        self.width = texture.width();
        self.height = texture.height();
        self.texture = Some(texture);

        Ok(())
    }

    pub fn get_color(&self, uv: Vector2<f64>) -> Vector3<f64> {
        let (w, h) = (self.width - 1, self.height - 1);

        let (x, y) = (uv.x % 1.0, uv.y % 1.0);
        let x = if x < 0.0 { x + 1.0 } else { x };
        let y = if y < 0.0 { y + 1.0 } else { y };

        let (x, y) = (x * f64::from(w), (1.0 - y) * f64::from(h));
        let (x, y) = (clamp(x as u32, 0, w), clamp(y as u32, 0, h));

        let pixel = self
            .texture
            .as_ref()
            .expect("texture not loaded")
            .get_pixel(x, y);
        let channels = pixel.channels();

        let norm = f64::from(std::u8::MAX);
        Vector3::new(
            f64::from(channels[0]) / norm,
            f64::from(channels[1]) / norm,
            f64::from(channels[2]) / norm,
        )
    }
}
