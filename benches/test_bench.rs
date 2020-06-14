use criterion::{criterion_group, criterion_main, Criterion};
use nalgebra::Point3;
use raytrace::{Camera, RenderOptions, Scene};
use std::fs::File;
use std::path::Path;
use std::time::Duration;

pub fn criterion_benchmark(c: &mut Criterion) {
    let scene = Scene::new(
        RenderOptions {
            width: 200,
            height: 200,
            max_depth: 5,
            ..RenderOptions::default()
        },
        Camera {
            position: Point3::from([2.0, 5.0, 15.0]),
            target: Point3::from([-1.0, 0.0, 0.0]),
            ..Camera::default()
        },
    );
    let empty_scene = scene.build_raytracing_scene();

    let scene_path = Path::new("scenes/benchmarks/simple.json");
    let scene_file = File::open(scene_path).expect("file not found");
    let mut scene: Scene = serde_json::from_reader(scene_file).expect("failed to parse scene");
    scene.load_assets(scene_path.parent().unwrap_or_else(|| Path::new("")));
    let simple_scene = scene.build_raytracing_scene();

    let scene_path = Path::new("scenes/benchmarks/complex.json");
    let scene_file = File::open(scene_path).expect("file not found");
    let mut scene: Scene = serde_json::from_reader(scene_file).expect("failed to parse scene");
    scene.load_assets(scene_path.parent().unwrap_or_else(|| Path::new("")));
    let complex_scene = scene.build_raytracing_scene();

    c.benchmark_group("raytrace")
        .sample_size(25)
        .measurement_time(Duration::new(10, 0))
        .bench_function("render empty scene", |b| {
            b.iter(|| empty_scene.raytrace_to_image(None))
        });

    c.benchmark_group("raytrace")
        .sample_size(25)
        .measurement_time(Duration::new(10, 0))
        .bench_function("render simple scene", |b| {
            b.iter(|| simple_scene.raytrace_to_image(None))
        });

    c.benchmark_group("raytrace")
        .sample_size(25)
        .measurement_time(Duration::new(45, 0))
        .bench_function("render complex scene", |b| {
            b.iter(|| complex_scene.raytrace_to_image(None))
        });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
