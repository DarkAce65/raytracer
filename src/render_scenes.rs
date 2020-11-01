#![deny(clippy::all)]

use raytrace::Scene;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn main() {
    let scenes = [
        "scenes/scene.json",
        "scenes/mesh.json",
        "scenes/refraction.json",
        "scenes/physical.json",
        "scenes/mesh_test.json",
        "scenes/occlusion.json",
        "scenes/test.json",
    ];
    let output_dir = "renders";
    let iterations = 3;

    for scene in &scenes {
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

        println!(
            "Raytracing {} ({} primitives)...",
            scene_path.display(),
            scene.get_num_objects()
        );
        for i in 0..iterations {
            print!("\u{2514} Iteration {}: tracing...", i + 1);
            io::stdout().flush().unwrap();

            let (image, duration, stats) = scene.raytrace_to_image(false);
            duration_sum += duration;
            ray_count_sum += stats.ray_count;

            println!(
                "\r\u{2502} Iteration {}: rendered in {:.3?} ({} rays)",
                i + 1,
                duration,
                stats.ray_count
            );

            if i == iterations - 1 {
                println!("\u{2502}\n\u{2502} Final render: {} x {} pixels, {} spp, {} primitives, {} rays, {:.3?}",
                    scene.get_width(),
                    scene.get_height(),
                    scene.render_options.samples_per_pixel,
                    scene.get_num_objects(),
                    ray_count_sum / iterations as u64,
                    duration_sum / iterations
                );
                image.save(&output_filename).expect("unable to write image");
                println!(
                    "\u{2514} Wrote rendered image to {}",
                    output_filename.display()
                );
            }
        }
        println!();
    }
}
