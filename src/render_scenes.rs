#![deny(clippy::all)]

use raytrace::Scene;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn main() {
    let scenes = [
        "scenes/scene.json",
        "scenes/physical.json",
        "scenes/refraction.json",
        "scenes/mesh.json",
        "scenes/mesh_test.json",
    ];
    let output_dir = "renders";
    let iterations = 3;

    for scene in scenes.iter() {
        let scene_path = Path::new(scene);
        let scene_file = File::open(scene_path).expect("file not found");

        let mut output_filename = PathBuf::from(output_dir);
        output_filename.push(scene_path.file_stem().unwrap());
        output_filename.set_extension("png");

        let mut scene: Scene = serde_json::from_reader(scene_file).expect("failed to parse scene");
        scene.load_assets(scene_path.parent().unwrap_or_else(|| Path::new("")));
        let scene = scene.build_raytracing_scene();

        let mut duration_sum = Duration::new(0, 0);
        let mut ray_count_sum = 0;

        println!("Raytracing {}...", scene_path.display());
        for i in 0..iterations {
            print!("└ Iteration {}: tracing...", i + 1);
            io::stdout().flush().unwrap();

            let (image, duration, ray_count) = scene.raytrace_to_image(None);
            duration_sum += duration;
            ray_count_sum += ray_count;

            println!(
                "\r│ Iteration {}: rendered in {:.3?} ({} rays)",
                i + 1,
                duration,
                ray_count
            );

            if i == iterations - 1 {
                println!(
                    "│ Avg time: {:.3?} (avg {} rays)",
                    duration_sum / iterations,
                    ray_count_sum / iterations as u64
                );
                image.save(&output_filename).expect("unable to write image");
                println!("└ Wrote rendered image to {}", output_filename.display());
            }
        }
        println!();
    }
}
