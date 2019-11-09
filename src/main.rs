extern crate image;

use image::{Pixel, RgbaImage};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

fn main() {
    let mut image = RgbaImage::new(WIDTH, HEIGHT);
    for (x, y, color) in image.enumerate_pixels_mut() {
        let channels = color.channels_mut();
        channels[0] = (255 * x / WIDTH) as u8;
        channels[1] = (255 * y / WIDTH) as u8;
        channels[3] = 255;
    }

    image.save("output.png").expect("Unable to write image");
}
