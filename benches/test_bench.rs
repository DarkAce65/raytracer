use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use nalgebra::Point3;
use raytrace::{Camera, RenderOptions, Scene};
use std::fmt;
use std::fs::File;
use std::path::Path;

struct Coordinates(u32, u32);

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.0, self.1)
    }
}

static COORDINATES: [Coordinates; 4] = [
    Coordinates(50, 50),
    Coordinates(50, 150),
    Coordinates(150, 50),
    Coordinates(150, 150),
];

pub fn empty_scene_benchmark(c: &mut Criterion) {
    let mut scene = Scene::new(
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
    scene.load_assets(Path::new(""));
    let empty_scene = scene.build_raytracing_scene();

    let mut group = c.benchmark_group("Empty scene");
    for coordinates in &COORDINATES {
        group.bench_with_input(
            BenchmarkId::new("Raycast", coordinates),
            coordinates,
            |b, c| b.iter(|| empty_scene.screen_raycast(c.0, c.1)),
        );
    }
    group.finish();
}

pub fn simple_scene_benchmark(c: &mut Criterion) {
    let scene_path = Path::new("scenes/benchmarks/simple.json");
    let scene_file = File::open(scene_path).expect("file not found");
    let mut scene: Scene = serde_json::from_reader(scene_file).expect("failed to parse scene");
    scene.load_assets(scene_path.parent().unwrap_or_else(|| Path::new("")));
    let simple_scene = scene.build_raytracing_scene();

    let mut group = c.benchmark_group("Simple scene");
    for coordinates in &COORDINATES {
        group.bench_with_input(
            BenchmarkId::new("Raycast", coordinates),
            coordinates,
            |b, c| b.iter(|| simple_scene.screen_raycast(c.0, c.1)),
        );
    }
    group.finish();
}

pub fn complex_scene_benchmark(c: &mut Criterion) {
    let scene_path = Path::new("scenes/benchmarks/complex.json");
    let scene_file = File::open(scene_path).expect("file not found");
    let mut scene: Scene = serde_json::from_reader(scene_file).expect("failed to parse scene");
    scene.load_assets(scene_path.parent().unwrap_or_else(|| Path::new("")));
    let complex_scene = scene.build_raytracing_scene();

    let mut group = c.benchmark_group("Complex scene");
    for coordinates in &COORDINATES {
        group.bench_with_input(
            BenchmarkId::new("Raycast", coordinates),
            coordinates,
            |b, c| b.iter(|| complex_scene.screen_raycast(c.0, c.1)),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    empty_scene_benchmark,
    simple_scene_benchmark,
    complex_scene_benchmark
);
criterion_main!(benches);
