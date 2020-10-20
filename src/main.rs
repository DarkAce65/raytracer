#![deny(clippy::all)]

use clap::{App, Arg};
use raytrace::Scene;
use std::fs::File;
use std::path::Path;
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
    let use_progress = !matches.is_present("noprogress");

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

    if let Some(filename) = output_filename {
        let (image, duration, _) = scene.raytrace_to_image(use_progress);
        image.save(filename).expect("unable to write image");
        println!("Output written to {} in {:.3?}", filename, duration);
    } else {
        scene.raytrace_to_buffer(use_progress);
    }
}
