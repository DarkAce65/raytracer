#![deny(clippy::all)]

use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use minifb::{Key, Window, WindowOptions};
use raytrace::Scene;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Instant;

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
        .get_matches();

    let scene_path = Path::new(matches.value_of("scene").unwrap());
    let scene_file = File::open(scene_path).expect("file not found");
    let output_filename = matches.value_of("output");
    let hide_progress = matches.is_present("noprogress");

    let mut scene: Scene = serde_json::from_reader(scene_file).expect("failed to parse scene");

    let now = Instant::now();
    scene.load_assets(scene_path.parent().unwrap_or_else(|| Path::new("")));
    println!("Took {:?} to load assets.", now.elapsed());

    let now = Instant::now();
    let scene = scene.build_raytracing_scene();
    println!(
        "Took {:?} to pre-process scene and construct bounding boxes for {} primitives.",
        now.elapsed(),
        scene.get_num_objects()
    );

    let (width, height) = (scene.get_width(), scene.get_height());

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
        println!("Raytracing...");
        let (image, duration, _) = scene.raytrace_to_image(progress);
        image.save(filename).expect("unable to write image");
        println!("Output written to {} in {:.3?}", filename, duration);
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
        scene.raytrace_to_buffer(&buffer_mutex, progress);

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
