extern crate clap;
extern crate image;
extern crate minifb;

use clap::{App, Arg};
use image::{Pixel, RgbaImage};
use minifb::{Key, Window, WindowOptions};
use std::convert::TryInto;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

fn main() {
    let matches = App::new("raytracer")
        .arg(
            Arg::with_name("framebuffer")
                .short("f")
                .long("framebuffer")
                .help("Runs the raytracer in a window"),
        )
        .get_matches();

    let use_framebuffer = matches.is_present("framebuffer");

    let mut image = RgbaImage::new(WIDTH, HEIGHT);
    for (x, y, color) in image.enumerate_pixels_mut() {
        let channels = color.channels_mut();
        channels[0] = (255 * x / WIDTH) as u8;
        channels[1] = (255 * y / WIDTH) as u8;
        channels[3] = 255;
    }

    if use_framebuffer {
        let w = WIDTH as usize;
        let h = HEIGHT as usize;

        let mut buffer: Vec<u32> = Vec::new();
        for pixel in image.into_vec().chunks_exact(4) {
            buffer.push(u32::from_le_bytes(pixel.try_into().unwrap()));
        }

        let mut window: Window = Window::new("raytracer", w, h, WindowOptions::default())
            .unwrap_or_else(|e| {
                panic!("{}", e);
            });

        println!("Rendering to window. Press escape to exit");
        while window.is_open() && !window.is_key_down(Key::Escape) {
            window.update_with_buffer(&buffer).unwrap();
        }
    } else {
        let filename = "output.png";
        image.save(filename).expect("Unable to write image");
        println!("Output written to {}", filename);
    }
}
