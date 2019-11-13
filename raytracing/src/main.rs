mod primitives;
mod raytrace;

use clap::{App, Arg};
use image::RgbaImage;
use minifb::{Key, Window, WindowOptions};
use nalgebra::Vector3;
use primitives::{Primitive, Sphere};
use raytrace::Ray;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;
const WIDTH_F: f32 = WIDTH as f32;
const HEIGHT_F: f32 = HEIGHT as f32;

pub struct Scene {
    objects: Vec<Box<dyn Primitive>>,
}

fn to_argb_u32(rgba: [u8; 4]) -> u32 {
    let (r, g, b, a) = (
        rgba[0] as u32,
        rgba[1] as u32,
        rgba[2] as u32,
        rgba[3] as u32,
    );
    a << 24 | r << 16 | g << 8 | b
}

fn raycast(scene: &Scene, x: u32, y: u32) -> [u8; 4] {
    let ray = Ray {
        origin: Vector3::from([x as f32, y as f32, 0.0]),
        direction: Vector3::from([0.0, 0.0, -1.0]),
    };

    let mut color = [0; 4];

    for object in scene.objects.iter() {
        if object.intersects(&ray) {
            color = object.color();
        }
    }

    color
}

fn raytrace_fb(scene: &Scene, image_buffer: &mut Vec<u32>, width: u32) {
    for (index, pixel) in image_buffer.iter_mut().enumerate() {
        let i = index as u32;
        *pixel = to_argb_u32(raycast(&scene, i % width, i / width));
    }
}

fn raytrace(scene: &Scene, image_buffer: &mut Vec<u8>, width: u32, height: u32) {
    for index in 0..width * height {
        let color = raycast(&scene, index % width, index / width);

        let index = (index * 4) as usize;
        image_buffer[index] = color[0];
        image_buffer[index + 1] = color[1];
        image_buffer[index + 2] = color[2];
        image_buffer[index + 3] = color[3];
    }
}

fn main() {
    let matches = App::new("raytracer")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("Output raytracer image to file"),
        )
        .get_matches();

    let output_filename = matches.value_of("file");

    let center = Vector3::from([WIDTH_F / 2.0, HEIGHT_F / 2.0, 0.0]);

    let mut scene = Scene {
        objects: Vec::new(),
    };
    scene.objects.push(Box::new(Sphere::from(10.0, center)));
    scene.objects.push(Box::new(Sphere::from(
        30.0,
        center + Vector3::from([30.0, -20.0, 45.0]),
    )));

    if output_filename.is_some() {
        let mut image_buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 4) as usize];
        raytrace(&scene, &mut image_buffer, WIDTH, HEIGHT);

        let filename = output_filename.unwrap();
        let image =
            RgbaImage::from_raw(WIDTH, HEIGHT, image_buffer).expect("Failed to convert buffer");
        image.save(filename).expect("Unable to write image");
        println!("Output written to {}", filename);

        return;
    }

    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    raytrace_fb(&scene, &mut buffer, WIDTH);

    let mut window: Window = Window::new(
        "raytracer",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    println!("Rendering to window. Press escape to exit");
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer).unwrap();
    }
}
