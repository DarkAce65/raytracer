#![warn(clippy::all)]

mod core;
mod lights;
mod object3d;
mod primitives;
mod ray_intersection;
mod scene;

use crate::scene::Scene;
use clap::{App, Arg};
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use minifb::{Key, Window, WindowOptions};
use nalgebra::Vector4;
use rand::{seq::SliceRandom, thread_rng};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

fn to_argb_u32(rgba: Vector4<f64>) -> u32 {
    let (r, g, b, a) = (
        (rgba.x.max(0.0).min(1.0) * 255.0) as u32,
        (rgba.y.max(0.0).min(1.0) * 255.0) as u32,
        (rgba.z.max(0.0).min(1.0) * 255.0) as u32,
        (rgba.w.max(0.0).min(1.0) * 255.0) as u32,
    );
    a << 24 | r << 16 | g << 8 | b
}

fn raytrace_fb(
    scene: Scene,
    buffer_mutex: &Arc<Mutex<Vec<u32>>>,
    progress: Option<ProgressBar>,
    render_sequentially: bool,
) {
    let buffer_mutex = Arc::clone(&buffer_mutex);
    let mut indexes: Vec<u32> = (0..scene.width * scene.height).collect();
    if !render_sequentially {
        indexes.shuffle(&mut thread_rng());
    }

    spawn(move || {
        let mut rays = 0;
        let iter: Box<dyn Iterator<Item = &u32>> = if let Some(progress) = &progress {
            Box::new(progress.wrap_iter(indexes.iter()))
        } else {
            Box::new(indexes.iter())
        };

        println!("Raytracing...");
        for index in iter {
            let (color, r) = scene.screen_raycast(*index);
            rays += r;
            let index = *index as usize;
            let mut buffer = buffer_mutex.lock().unwrap();
            buffer[index] = to_argb_u32(color);
            drop(buffer);

            if let Some(progress) = &progress {
                progress.set_message(&rays.to_string());
            }
        }

        if let Some(progress) = progress {
            progress.finish();
        }
    });
}

fn raytrace(scene: &Scene, image_buffer: &mut Vec<u8>, progress: Option<ProgressBar>) -> Duration {
    let mut indexes: Vec<u32> = (0..scene.width * scene.height).collect();
    indexes.shuffle(&mut thread_rng());

    let mut rays = 0;
    let iter: Box<dyn Iterator<Item = &u32>> = if let Some(progress) = &progress {
        Box::new(progress.wrap_iter(indexes.iter()))
    } else {
        Box::new(indexes.iter())
    };

    println!("Raytracing...");
    let start = Instant::now();
    for index in iter {
        let (color, r) = scene.screen_raycast(*index);
        rays += r;

        let index = (index * 4) as usize;
        image_buffer[index] = (color.x * 255.0) as u8;
        image_buffer[index + 1] = (color.y * 255.0) as u8;
        image_buffer[index + 2] = (color.z * 255.0) as u8;
        image_buffer[index + 3] = (color.w * 255.0) as u8;

        if let Some(progress) = &progress {
            progress.set_message(&rays.to_string());
        }
    }

    if let Some(progress) = progress {
        progress.finish();
    }

    start.elapsed()
}

fn main() {
    let matches = App::new("ray tracer")
        .about("A ray tracer written in Rust")
        .arg(
            Arg::with_name("scene")
                .index(1)
                .required(true)
                .takes_value(true)
                .help("input scene as a json file"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help(
                    "Output rendered image to file\n\
                     If omitted, image is rendered to a window",
                ),
        )
        .arg(
            Arg::with_name("noprogress")
                .long("no-progress")
                .help("Hide progress bar"),
        )
        .arg(Arg::with_name("norandom").long("no-random").help(
            "Render to window sequentially instead of randomly\n\
             If --output is specified, --no-random has no effect",
        ))
        .get_matches();

    let scene_path = Path::new(matches.value_of("scene").unwrap());
    let scene_file = File::open(scene_path).expect("File not found");
    let output_filename = matches.value_of("output");
    let hide_progress = matches.is_present("noprogress");
    let render_sequentially = matches.is_present("norandom");

    let mut scene: Scene = serde_json::from_reader(scene_file).expect("Failed to parse scene");
    scene.load_assets(scene_path.parent().unwrap_or_else(|| Path::new("")));
    let (width, height) = (scene.width, scene.height);

    let progress = if hide_progress {
        None
    } else {
        let progress = ProgressBar::new((width * height).into());
        progress.set_draw_delta((width * height / 200).into());
        progress.set_style(ProgressStyle::default_bar().template(
            "[{elapsed_precise} elapsed] [{eta_precise} left] \
             {bar:40} {pos}/{len} pixels, {msg} rays",
        ));
        Some(progress)
    };

    if let Some(filename) = output_filename {
        let mut image_buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

        let duration = raytrace(&scene, &mut image_buffer, progress);

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
    raytrace_fb(scene, &buffer_mutex, progress, render_sequentially);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let buffer = buffer_mutex.lock().unwrap();
        window.update_with_buffer(&buffer).unwrap();
        drop(buffer);
        sleep(std::time::Duration::from_millis(100));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_converts_color_vecs_to_u32() {
        let color = 0;
        assert_eq!(to_argb_u32(Vector4::from([0.0, 0.0, 0.0, 0.0])), color);
        let color = 255 << 24;
        assert_eq!(to_argb_u32(Vector4::from([0.0, 0.0, 0.0, 1.0])), color);
        let color = 255 << 24 | 255 << 16 | 255 << 8 | 255;
        assert_eq!(to_argb_u32(Vector4::from([1.0, 1.0, 1.0, 1.0])), color);
        let color = 255 << 24 | 255;
        assert_eq!(to_argb_u32(Vector4::from([0.0, 0.0, 1.0, 1.0])), color);
        let color = 255 << 24 | 255 << 16 | 255;
        assert_eq!(to_argb_u32(Vector4::from([1.0, -1.0, 2.0, 1.0])), color);
    }
}
