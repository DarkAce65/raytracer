#![warn(clippy::all)]

mod core;
mod lights;
mod primitives;
mod ray_intersection;
mod scene;

use crate::scene::{RaytracingScene, Scene};
use clap::{App, Arg};
use image::RgbaImage;
use indicatif::ParallelProgressIterator;
use indicatif::{ProgressBar, ProgressStyle};
use minifb::{Key, Window, WindowOptions};
use nalgebra::Vector4;
use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::*;
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

fn to_argb_u32(rgba: Vector4<f64>) -> u32 {
    let (r, g, b, a) = (
        (rgba.x * 255.0) as u32,
        (rgba.y * 255.0) as u32,
        (rgba.z * 255.0) as u32,
        (rgba.w * 255.0) as u32,
    );
    a << 24 | r << 16 | g << 8 | b
}

fn raytrace_fb(
    scene: RaytracingScene,
    buffer_mutex: &Arc<Mutex<Vec<u32>>>,
    progress: Option<ProgressBar>,
    render_sequentially: bool,
) {
    let buffer_mutex = Arc::clone(&buffer_mutex);
    let mut indexes: Vec<u32> = (0..scene.get_width() * scene.get_height()).collect();
    if !render_sequentially {
        indexes.shuffle(&mut thread_rng());
    }

    spawn(move || {
        let rays = AtomicU64::new(0);
        let width = scene.get_width();

        let process_pixel = |index| {
            let (color, r) = scene.screen_raycast(index % width, index / width);
            rays.fetch_add(r, Ordering::SeqCst);

            let mut buffer = buffer_mutex.lock().unwrap();
            buffer[index as usize] = to_argb_u32(color);
        };

        if let Some(progress) = progress {
            indexes
                .into_par_iter()
                .progress_with(progress.clone())
                .inspect(|_| progress.set_message(&rays.load(Ordering::SeqCst).to_string()))
                .for_each(process_pixel);

            progress.finish_with_message(&rays.load(Ordering::SeqCst).to_string());
        } else {
            indexes.into_par_iter().for_each(process_pixel);
        }
    });
}

fn raytrace(
    scene: RaytracingScene,
    image_buffer: &mut Vec<u8>,
    progress: Option<ProgressBar>,
) -> Duration {
    let mut indexes: Vec<u32> = (0..scene.get_width() * scene.get_height()).collect();
    indexes.shuffle(&mut thread_rng());

    let buffer_mutex = Arc::new(Mutex::new(image_buffer));
    let rays = AtomicU64::new(0);
    let width = scene.get_width();

    let process_pixel = |index| {
        let (color, r) = scene.screen_raycast(index % width, index / width);
        rays.fetch_add(r, Ordering::SeqCst);

        let index = (index * 4) as usize;
        let mut buffer = buffer_mutex.lock().unwrap();
        buffer[index] = (color.x * 255.0) as u8;
        buffer[index + 1] = (color.y * 255.0) as u8;
        buffer[index + 2] = (color.z * 255.0) as u8;
        buffer[index + 3] = (color.w * 255.0) as u8;
    };

    let start = Instant::now();

    if let Some(progress) = progress {
        indexes
            .into_par_iter()
            .progress_with(progress.clone())
            .inspect(|_| progress.set_message(&rays.load(Ordering::SeqCst).to_string()))
            .for_each(process_pixel);

        progress.finish_with_message(&rays.load(Ordering::SeqCst).to_string());
    } else {
        indexes.into_par_iter().for_each(process_pixel);
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
    let scene_file = File::open(scene_path).expect("file not found");
    let output_filename = matches.value_of("output");
    let hide_progress = matches.is_present("noprogress");
    let render_sequentially = matches.is_present("norandom");

    let mut scene: Scene = serde_json::from_reader(scene_file).expect("failed to parse scene");

    let now = Instant::now();
    scene.load_assets(scene_path.parent().unwrap_or_else(|| Path::new("")));
    println!("Took {:?} to load assets.", now.elapsed());

    let now = Instant::now();
    let scene = scene.build_raytracing_scene();
    println!(
        "Took {:?} to pre-process scene and construct bounding boxes.",
        now.elapsed()
    );

    let (width, height) = (scene.get_width(), scene.get_height());

    let progress = if hide_progress {
        None
    } else {
        let progress = ProgressBar::new((width * height).into());
        progress.set_draw_delta((width * height / 200).into());
        if render_sequentially {
            progress.set_style(ProgressStyle::default_bar().template(
                "[{elapsed_precise} elapsed] \
                 {bar:40} {pos}/{len} pixels, {msg} rays",
            ));
        } else {
            progress.set_style(ProgressStyle::default_bar().template(
                "[{elapsed_precise} elapsed] [{eta_precise} left] \
                 {bar:40} {pos}/{len} pixels, {msg} rays",
            ));
        }
        Some(progress)
    };

    if let Some(filename) = output_filename {
        let mut image_buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

        println!("Raytracing...");
        let duration = raytrace(scene, &mut image_buffer, progress);

        let image =
            RgbaImage::from_raw(width, height, image_buffer).expect("failed to convert buffer");
        image.save(filename).expect("unable to write image");
        println!("Output written to {} in {:?}", filename, duration);
    } else {
        println!("Rendering to window - press escape to exit.");
        let mut window: Window = Window::new(
            "raytracer",
            width as usize,
            height as usize,
            WindowOptions {
                title: false,
                borderless: true,
                ..WindowOptions::default()
            },
        )
        .unwrap();

        let image_buffer: Vec<u32> = vec![0; (width * height) as usize];
        let buffer_mutex = Arc::new(Mutex::new(image_buffer));

        println!("Raytracing...");
        raytrace_fb(scene, &buffer_mutex, progress, render_sequentially);

        while window.is_open() && !window.is_key_down(Key::Escape) {
            {
                let buffer = buffer_mutex.lock().unwrap();
                window
                    .update_with_buffer(&buffer, width as usize, height as usize)
                    .unwrap();
            }
            sleep(std::time::Duration::from_millis(100));
        }
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
