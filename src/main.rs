mod primitives;
mod raytrace;

use clap::{App, Arg};
use image::{Rgba, RgbaImage};
use minifb::{Key, Window, WindowOptions};
use nalgebra::Vector3;
use primitives::{Intersectable, Sphere};
use raytrace::Ray;
use std::convert::TryInto;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

pub struct Scene {
    objects: Vec<Box<dyn Intersectable>>,
}

fn raycast(scene: &Scene, x: f32, y: f32) -> Rgba<u8> {
    let ray = Ray {
        origin: Vector3::from([x, y, 0.0]),
        direction: Vector3::from([0.0, 0.0, -1.0]),
    };

    let mut r: u8 = 0;
    let mut g: u8 = 0;
    let mut b: u8 = 0;
    let a: u8 = 255;

    for object in scene.objects.iter() {
        if object.intersects(&ray) {
            r = 255;
            g = 255;
            b = 255;
        }
    }

    Rgba([b, g, r, a])
}

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

    let center = Vector3::from([WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0, 0.0]);

    let mut scene = Scene {
        objects: Vec::new(),
    };
    scene.objects.push(Box::new(Sphere::from(10.0, center)));
    scene.objects.push(Box::new(Sphere::from(
        30.0,
        center + Vector3::from([30.0, -20.0, 45.0]),
    )));

    let mut image = RgbaImage::new(WIDTH, HEIGHT);
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            image.put_pixel(x, y, raycast(&scene, x as f32, y as f32));
        }
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
