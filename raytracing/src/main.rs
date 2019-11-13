mod primitives;
mod raytrace;

use clap::{App, Arg};
use image::RgbaImage;
use minifb::{Key, Window, WindowOptions};
use nalgebra::Vector3;
use primitives::SphereBuilder;
use rand::{seq::SliceRandom, thread_rng};
use raytrace::{raycast, Scene};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Instant;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const WIDTH_F: f32 = WIDTH as f32;
const HEIGHT_F: f32 = HEIGHT as f32;

fn to_argb_u32(rgba: [u8; 4]) -> u32 {
    let (r, g, b, a) = (
        rgba[0] as u32,
        rgba[1] as u32,
        rgba[2] as u32,
        rgba[3] as u32,
    );
    a << 24 | r << 16 | g << 8 | b
}

fn raytrace_fb(scene: Scene, buffer_mutex: &Arc<Mutex<Vec<u32>>>, width: u32, height: u32) {
    let buffer_mutex = Arc::clone(&buffer_mutex);
    let mut indexes: Vec<u32> = (0..width * height).collect();
    indexes.shuffle(&mut thread_rng());

    println!("Raytracing...");
    spawn(move || {
        for index in indexes.iter() {
            let color = raycast(&scene, index % width, index / width);
            let index = *index as usize;
            let mut buffer = buffer_mutex.lock().unwrap();
            buffer[index] = to_argb_u32(color);
            drop(buffer);
        }

        println!("Done.");
    });
}

fn raytrace(scene: Scene, image_buffer: &mut Vec<u8>, width: u32, height: u32) {
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
    scene.objects.push(Box::new(
        SphereBuilder::default().center(center).build().unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(30.)
            .center(center + Vector3::from([30.0, -20.0, 45.0]))
            .color([0, 64, 127, 255])
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(70.0)
            .center(center + Vector3::from([-80.0, 80.0, 0.0]))
            .color([255, 0, 0, 255])
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(90.0)
            .center(center + Vector3::from([220.0, 190.0, 0.0]))
            .color([0, 64, 0, 255])
            .build()
            .unwrap(),
    ));

    if output_filename.is_some() {
        let mut image_buffer: Vec<u8> = vec![0; (WIDTH * HEIGHT * 4) as usize];

        let start = Instant::now();
        raytrace(scene, &mut image_buffer, WIDTH, HEIGHT);
        let duration = start.elapsed();

        let filename = output_filename.unwrap();
        let image =
            RgbaImage::from_raw(WIDTH, HEIGHT, image_buffer).expect("Failed to convert buffer");
        image.save(filename).expect("Unable to write image");
        println!("Output written to {} in {:?}", filename, duration);

        return;
    }

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

    let image_buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];
    let buffer_mutex = Arc::new(Mutex::new(image_buffer));
    raytrace_fb(scene, &buffer_mutex, WIDTH, HEIGHT);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let buffer = buffer_mutex.lock().unwrap();
        window.update_with_buffer(&buffer).unwrap();
        drop(buffer);
        sleep(std::time::Duration::from_millis(16));
    }
}
