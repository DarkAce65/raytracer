#![warn(clippy::all)]

mod primitives;
mod raytrace;

use clap::{App, Arg};
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use minifb::{Key, Window, WindowOptions};
use nalgebra::{clamp, Vector3, Vector4};
use primitives::SphereBuilder;
use rand::{seq::SliceRandom, thread_rng};
use raytrace::{raycast, Scene};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Instant;

fn to_argb_u32(rgba: Vector4<f32>) -> u32 {
    let rgba = rgba.map(|c| clamp(c, 0.0, 1.0));
    let (r, g, b, a) = (
        (rgba.x * 255.0) as u32,
        (rgba.y * 255.0) as u32,
        (rgba.z * 255.0) as u32,
        (rgba.w * 255.0) as u32,
    );
    a << 24 | r << 16 | g << 8 | b
}

fn to_xy(scene: &Scene, index: u32) -> (f32, f32) {
    let (w, h) = (scene.width as f32, scene.height as f32);
    let (x, y) = ((index % scene.width) as f32, (index / scene.width) as f32);
    let aspect = w / h;
    let fov = (scene.fov.to_radians() / 2.0).tan();
    let x = (((x + 0.5) / w) * 2.0 - 1.0) * fov;
    let y = (1.0 - ((y + 0.5) / h) * 2.0) * fov;

    if scene.width < scene.height {
        return (x * aspect, y);
    }

    (x, y / aspect)
}

fn raytrace_fb(scene: Scene, buffer_mutex: &Arc<Mutex<Vec<u32>>>, progress: Option<ProgressBar>) {
    let buffer_mutex = Arc::clone(&buffer_mutex);
    let mut indexes: Vec<u32> = (0..scene.width * scene.height).collect();
    indexes.shuffle(&mut thread_rng());

    spawn(move || {
        println!("Raytracing...");
        for index in indexes.iter() {
            if let Some(progress) = &progress {
                progress.inc(1);
            }

            let (x, y) = to_xy(&scene, *index);
            let color = raycast(&scene, x, y);
            let index = *index as usize;
            let mut buffer = buffer_mutex.lock().unwrap();
            buffer[index] = to_argb_u32(color);
            drop(buffer);
        }

        if let Some(progress) = progress {
            progress.finish();
        }
    });
}

fn raytrace(scene: &Scene, image_buffer: &mut Vec<u8>, progress: Option<ProgressBar>) {
    println!("Raytracing...");
    for index in 0..scene.width * scene.height {
        if let Some(progress) = &progress {
            progress.inc(1);
        }

        let (x, y) = to_xy(&scene, index);
        let color = raycast(&scene, x, y);
        let color = color.map(|c| clamp(c, 0.0, 1.0));

        let index = (index * 4) as usize;
        image_buffer[index] = (color.x * 255.0) as u8;
        image_buffer[index + 1] = (color.y * 255.0) as u8;
        image_buffer[index + 2] = (color.z * 255.0) as u8;
        image_buffer[index + 3] = (color.w * 255.0) as u8;
    }

    if let Some(progress) = progress {
        progress.finish();
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
        .arg(
            Arg::with_name("noprogress")
                .long("no-progress")
                .help("Hide progress bar"),
        )
        .get_matches();

    let output_filename = matches.value_of("file");
    let hide_progress = matches.is_present("noprogress");

    let mut scene: Scene = Scene {
        width: 800,
        height: 800,
        fov: 65.0,
        objects: Vec::new(),
    };
    let (width, height) = (scene.width, scene.height);
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .center(Vector3::from([0.0, 0.0, -5.0]))
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(3.0)
            .center(Vector3::from([3.0, -2.0, -5.0]))
            .color(Vector4::from([0.0, 0.25, 0.5, 1.0]))
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(7.0)
            .center(Vector3::from([-6.0, 6.0, -18.0]))
            .color(Vector4::from([1.0, 0.25, 0.1, 1.0]))
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(9.0)
            .center(Vector3::from([22.0, 5.0, -100.0]))
            .color(Vector4::from([0.1, 0.5, 0.1, 1.0]))
            .build()
            .unwrap(),
    ));

    let progress = if hide_progress {
        None
    } else {
        let progress = ProgressBar::new((width * height).into());
        progress.set_draw_delta((width * height / 200).into());
        progress.set_style(
            ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40} {pos}/{len} rays"),
        );
        Some(progress)
    };

    if let Some(filename) = output_filename {
        let mut image_buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

        let start = Instant::now();
        raytrace(&scene, &mut image_buffer, progress);
        let duration = start.elapsed();

        let image =
            RgbaImage::from_raw(width, height, image_buffer).expect("Failed to convert buffer");
        image.save(filename).expect("Unable to write image");
        println!("Output written to {} in {:?}", filename, duration);

        return;
    }

    let mut window: Window = Window::new(
        "raytracer",
        width as usize,
        height as usize,
        WindowOptions {
            borderless: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    println!("Rendering to window. Press escape to exit");

    let image_buffer: Vec<u32> = vec![0; (width * height) as usize];
    let buffer_mutex = Arc::new(Mutex::new(image_buffer));
    raytrace_fb(scene, &buffer_mutex, progress);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let buffer = buffer_mutex.lock().unwrap();
        window.update_with_buffer(&buffer).unwrap();
        drop(buffer);
        sleep(std::time::Duration::from_millis(100));
    }
}
