#![warn(clippy::all)]

mod core;
mod lights;
mod primitives;
mod scene;

use clap::{App, Arg};
use image::RgbaImage;
use indicatif::{ProgressBar, ProgressStyle};
use lights::{AmbientLightBuilder, PointLightBuilder};
use minifb::{Key, Window, WindowOptions};
use nalgebra::{clamp, Point3, Vector3, Vector4};
use primitives::{CubeBuilder, MaterialBuilder, MaterialSide, SphereBuilder};
use rand::{seq::SliceRandom, thread_rng};
use scene::{Camera, Scene};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

fn to_argb_u32(rgba: Vector4<f64>) -> u32 {
    let rgba = rgba.map(|c| clamp(c, 0.0, 1.0));
    let (r, g, b, a) = (
        (rgba.x * 255.0) as u32,
        (rgba.y * 255.0) as u32,
        (rgba.z * 255.0) as u32,
        (rgba.w * 255.0) as u32,
    );
    a << 24 | r << 16 | g << 8 | b
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

            let color = scene.screen_raycast(*index);
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

fn raytrace(scene: &Scene, image_buffer: &mut Vec<u8>, progress: Option<ProgressBar>) -> Duration {
    let mut indexes: Vec<u32> = (0..scene.width * scene.height).collect();
    indexes.shuffle(&mut thread_rng());

    println!("Raytracing...");
    let start = Instant::now();
    for index in indexes.iter() {
        if let Some(progress) = &progress {
            progress.inc(1);
        }

        let color = scene.screen_raycast(*index);
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

    start.elapsed()
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
        camera: Camera::from(
            65.0,
            Point3::from([2.0, 5.0, 15.0]),
            Point3::origin(),
            Vector3::y_axis(),
        ),
        lights: Vec::new(),
        objects: Vec::new(),
    };
    let (width, height) = (scene.width, scene.height);
    scene.lights.push(Box::new(
        AmbientLightBuilder::default()
            .color(Vector3::from([0.125; 3]))
            .build()
            .unwrap(),
    ));
    scene.lights.push(Box::new(
        PointLightBuilder::default()
            .position(Point3::from([-8.0, 3.0, 0.0]))
            .color(Vector3::from([0.5, 0.5, 0.5]))
            .build()
            .unwrap(),
    ));
    scene.lights.push(Box::new(
        PointLightBuilder::default()
            .position(Point3::from([-2.0, 5.0, -10.0]))
            .color(Vector3::from([0.5, 0.0, 0.0]))
            .build()
            .unwrap(),
    ));
    scene.lights.push(Box::new(
        PointLightBuilder::default()
            .position(Point3::from([3.0, 5.0, -3.0]))
            .color(Vector3::from([0.0, 0.3, 0.5]))
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        CubeBuilder::default()
            .size(120.0)
            .center(Point3::from([30.0, 49.0, -40.0]))
            .material(
                MaterialBuilder::default()
                    .side(MaterialSide::Back)
                    .color(Vector3::from([0.6; 3]))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .center(Point3::from([0.0, 0.0, -5.0]))
            .material(
                MaterialBuilder::default()
                    .color(Vector3::from([1.0; 3]))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(3.0)
            .center(Point3::from([3.0, -2.0, -5.0]))
            .material(
                MaterialBuilder::default()
                    .color(Vector3::from([0.0, 0.25, 0.5]))
                    .specular(Vector3::from([1.0; 3]))
                    .shininess(10.0)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(6.0)
            .center(Point3::from([-6.0, 6.0, -18.0]))
            .material(
                MaterialBuilder::default()
                    .color(Vector3::from([1.0, 0.25, 0.1]))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(4.0)
            .center(Point3::from([-6.0, -9.0, -3.0]))
            .material(
                MaterialBuilder::default()
                    .color(Vector3::from([0.4, 0.25, 0.6]))
                    .specular(Vector3::from([0.5; 3]))
                    .shininess(100.0)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(5.0)
            .center(Point3::from([-20.0, -9.0, -40.0]))
            .material(
                MaterialBuilder::default()
                    .color(Vector3::from([0.1, 0.5, 0.1]))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        SphereBuilder::default()
            .radius(1.5)
            .center(Point3::from([2.0, -10.0, -2.0]))
            .material(
                MaterialBuilder::default()
                    .emissive(Vector3::from([0.0, 1.0, 0.0]))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        CubeBuilder::default()
            .size(2.0)
            .center(Point3::from([-4.0, -5.0, -2.0]))
            .material(
                MaterialBuilder::default()
                    .color(Vector3::from([0.5, 0.1, 0.1]))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
    ));
    scene.objects.push(Box::new(
        CubeBuilder::default()
            .size(2.0)
            .center(Point3::from([1.0, -2.0, -2.0]))
            .material(
                MaterialBuilder::default()
                    .color(Vector3::from([0.9, 0.7, 0.1]))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap(),
    ));

    let progress = if hide_progress {
        None
    } else {
        let progress = ProgressBar::new((width * height).into());
        progress.set_draw_delta((width * height / 200).into());
        progress.set_style(ProgressStyle::default_bar().template(
            "[{elapsed_precise} elapsed] [{eta_precise} left] {bar:40} {pos}/{len} pixels",
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
    raytrace_fb(scene, &buffer_mutex, progress);

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
