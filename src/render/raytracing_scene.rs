use super::{Camera, CastStats, ColorData, RenderOptions, BIAS};
use crate::core::{
    KdTreeAccelerator, Material, PhongMaterial, PhysicalMaterial, Texture, Transformed,
};
use crate::lights::Light;
use crate::ray_intersection::{Intersection, Ray, RayType};
use crate::utils;
use image::RgbaImage;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use minifb::{Key, Window, WindowOptions};
use nalgebra::{Matrix4, Point3, Unit, Vector3};
use num_traits::identities::Zero;
use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::*;
use std::collections::HashMap;
use std::f64::consts::{FRAC_1_PI, FRAC_PI_2};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct RaytracingCamera {
    fov: f64,
    position: Point3<f64>,
    camera_to_world: Matrix4<f64>,
}

impl From<Camera> for RaytracingCamera {
    fn from(camera: Camera) -> Self {
        let camera_to_world =
            Matrix4::look_at_rh(&camera.position, &camera.target, &camera.up).transpose();

        Self {
            fov: camera.fov,
            position: camera.position,
            camera_to_world,
        }
    }
}

#[derive(Debug)]
pub struct RaytracingScene {
    pub render_options: RenderOptions,
    camera: RaytracingCamera,
    lights: Vec<Light>,
    textures: HashMap<String, Texture>,
    object_tree: KdTreeAccelerator,
}

impl RaytracingScene {
    pub fn new(
        render_options: RenderOptions,
        camera: RaytracingCamera,
        lights: Vec<Light>,
        textures: HashMap<String, Texture>,
        object_tree: KdTreeAccelerator,
    ) -> Self {
        Self {
            render_options,
            camera,
            lights,
            textures,
            object_tree,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.render_options.width
    }

    pub fn get_height(&self) -> u32 {
        self.render_options.height
    }

    fn get_aspect(&self) -> f64 {
        f64::from(self.get_width()) / f64::from(self.get_height())
    }

    fn compute_screen_to_fov(&self) -> f64 {
        (self.camera.fov.to_radians() / 2.0).tan()
    }

    pub fn get_num_objects(&self) -> usize {
        self.object_tree.get_num_objects()
    }

    fn raycast(&self, ray: &Ray) -> Option<Intersection> {
        self.object_tree.raycast(ray)
    }

    fn shadow_cast(&self, ray: &Ray, max_distance: f64) -> bool {
        self.object_tree.shadow_cast(ray, max_distance - BIAS)
    }

    fn get_color_phong(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        material: &PhongMaterial,
    ) -> (ColorData, CastStats) {
        let mut cast_stats = CastStats::zero();
        let depth = ray.get_depth();
        let hit_point = intersection.get_hit_point();

        let normal = intersection.get_normal();

        let uv = intersection.get_uv();
        let material_color = material.get_color(uv, &self.textures);
        let emissive = material.emissive;

        let reflection = if material.reflectivity > 0.0 {
            let reflection_dir = utils::reflect(&ray.direction, &normal).into_inner();
            let reflection_ray = Ray {
                ray_type: RayType::Secondary(depth + 1),
                origin: hit_point + (reflection_dir * BIAS),
                direction: reflection_dir,
                refractive_index: 1.0,
            };
            let (mut color_data, stats) = self.get_color(&reflection_ray);
            color_data.color.component_mul_assign(&material_color);
            cast_stats += stats;

            Some(color_data)
        } else {
            None
        };

        let mut ambient_light = Vector3::zero();
        let mut irradiance = Vector3::zero();
        if material.reflectivity < 1.0 {
            for light in &self.lights {
                match light {
                    Light::Ambient(light) => {
                        ambient_light += light.get_color().component_mul(&material_color);
                    }
                    Light::Point(light) => {
                        let light_position = light.get_position();
                        let light_dir = light_position - hit_point;
                        let light_distance = light_dir.magnitude();
                        let light_dir = light_dir.normalize();

                        let n_dot_l = normal.dot(&light_dir);
                        if n_dot_l > 0.0 {
                            let shadow_ray = Ray {
                                ray_type: RayType::Shadow,
                                origin: light_position,
                                direction: -light_dir,
                                refractive_index: 1.0,
                            };

                            cast_stats.ray_count += 1;
                            if !self.shadow_cast(&shadow_ray, light_distance) {
                                let light_color = light.get_color(light_distance);
                                irradiance += light_color.component_mul(&material_color) * n_dot_l;

                                let half_vec = Unit::new_normalize(light_dir - ray.direction);
                                let n_dot_h = normal.dot(&half_vec);
                                if n_dot_h > 0.0 {
                                    irradiance += light_color.component_mul(&material.specular)
                                        * n_dot_h.powf(material.shininess);
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut color_data = ColorData::new(ambient_light + irradiance, material_color, emissive);

        // Ambient occlusion computation can be skipped for perfectly reflective materials
        if material.reflectivity < 1.0 {
            let (ambient_occlusion, ambient_occlusion_stats) =
                self.compute_ambient_occlusion(intersection, depth);
            color_data.ambient_occlusion = ambient_occlusion;
            cast_stats += ambient_occlusion_stats;
        }

        if let Some(reflection) = reflection {
            color_data.color = color_data
                .color
                .lerp(&reflection.color, material.reflectivity);
            color_data.emissive = color_data
                .emissive
                .lerp(&reflection.emissive, material.reflectivity);
            color_data.ambient_occlusion = utils::lerp(
                color_data.ambient_occlusion,
                reflection.ambient_occlusion,
                material.reflectivity,
            );
        }

        (color_data, cast_stats)
    }

    fn get_color_physical(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        material: &PhysicalMaterial,
    ) -> (ColorData, CastStats) {
        let mut cast_stats = CastStats::zero();
        let depth = ray.get_depth();
        let hit_point = intersection.get_hit_point();

        let normal = intersection.get_normal();
        let view_dir = Unit::new_normalize(-ray.direction);
        let n_dot_v = normal.dot(&view_dir).max(0.0);

        let uv = intersection.get_uv();
        let material_color = material.get_color(uv, &self.textures);

        let roughness = material.roughness.max(0.04);
        let base_reflectivity = Vector3::repeat(0.04).lerp(&material_color, material.metalness);
        let f = utils::fresnel(n_dot_v, base_reflectivity);
        let k_s = f;
        let k_d = (Vector3::repeat(1.0) - k_s) * (1.0 - material.metalness);

        let emissive = material.emissive;

        let reflection = if self.render_options.max_reflected_rays > 0 {
            let d = 8_u16.pow(depth.into());
            let reflected_rays = (self.render_options.max_reflected_rays / d).max(1);

            let max_angle = FRAC_PI_2 * material.roughness;
            let reflection_dir = utils::reflect(&ray.direction, &normal);

            let mut reflection = (0..reflected_rays).fold(ColorData::zero(), |mut acc, _| {
                let direction = utils::uniform_sample_cone(&reflection_dir, max_angle).into_inner();
                let reflection_ray = Ray {
                    ray_type: RayType::Secondary(depth + 1),
                    origin: hit_point + (direction * BIAS),
                    direction,
                    refractive_index: 1.0,
                };
                let (color_data, stats) = self.get_color(&reflection_ray);
                cast_stats += stats;

                acc.color += color_data.color;
                acc.ambient_occlusion += color_data.ambient_occlusion;

                acc
            });
            reflection
                .color
                .component_mul_assign(&(f * FRAC_PI_2 / f64::from(reflected_rays)));
            reflection.ambient_occlusion /= f64::from(reflected_rays);

            Some(reflection)
        } else {
            None
        };

        let refraction = if material.opacity < 1.0 {
            let eta = ray.refractive_index / material.refractive_index;
            utils::refract(&ray.direction, &normal, eta).map(|refraction_dir| {
                let refraction_dir = refraction_dir.into_inner();
                let refraction_ray = Ray {
                    ray_type: RayType::Secondary(depth + 1),
                    origin: hit_point + (refraction_dir * BIAS),
                    direction: refraction_dir,
                    refractive_index: material.refractive_index,
                };
                let (color_data, stats) = self.get_color(&refraction_ray);
                cast_stats += stats;

                color_data.color.component_mul(&material_color)
            })
        } else {
            None
        };

        let mut ambient_light = Vector3::zero();
        let mut irradiance = Vector3::zero();
        let diffuse = FRAC_1_PI * k_d.component_mul(&material_color);
        for light in &self.lights {
            match light {
                Light::Ambient(light) => {
                    ambient_light += light.get_color().component_mul(&material_color);
                }
                Light::Point(light) => {
                    let light_position = light.get_position();
                    let light_dir = light_position - hit_point;
                    let light_distance = light_dir.magnitude();
                    let light_dir = light_dir.normalize();

                    let n_dot_l = normal.dot(&light_dir);
                    if n_dot_l > 0.0 {
                        let shadow_ray = Ray {
                            ray_type: RayType::Shadow,
                            origin: light_position,
                            direction: -light_dir,
                            refractive_index: 1.0,
                        };

                        cast_stats.ray_count += 1;
                        if !self.shadow_cast(&shadow_ray, light_distance) {
                            let half_vec = Unit::new_normalize(light_dir - ray.direction);
                            let n_dot_h = normal.dot(&half_vec).max(0.0);

                            let light_color = light.get_color(light_distance);
                            let radiance = light_color * n_dot_l;

                            let ndf = utils::ndf(n_dot_h, roughness);
                            let g = utils::geometry_function(n_dot_v, n_dot_l, roughness);

                            let diffuse_specular = if n_dot_v == 0.0 {
                                diffuse
                            } else {
                                let specular = ndf * g * f / (4.0 * n_dot_v * n_dot_l);
                                diffuse + specular
                            };

                            irradiance += diffuse_specular.component_mul(&radiance) * n_dot_l;
                        }
                    }
                }
            };
        }

        let mut color_data = ColorData::new(ambient_light + irradiance, material_color, emissive);

        let (ambient_occlusion, ambient_occlusion_stats) =
            self.compute_ambient_occlusion(intersection, depth);
        color_data.ambient_occlusion = ambient_occlusion;
        cast_stats += ambient_occlusion_stats;

        if let Some(reflection) = reflection {
            color_data.color += reflection.color;
            color_data.emissive += reflection.emissive;
            color_data.ambient_occlusion = utils::lerp(
                color_data.ambient_occlusion,
                reflection.ambient_occlusion,
                1.0 - roughness,
            );
        }

        if let Some(refraction) = refraction {
            color_data.color = color_data.color.lerp(&refraction, material.opacity);
            color_data.emissive = color_data.emissive.lerp(&refraction, material.opacity);
        }

        (color_data, cast_stats)
    }

    fn compute_ambient_occlusion(
        &self,
        intersection: &Intersection,
        depth: u8,
    ) -> (f64, CastStats) {
        let mut cast_stats = CastStats::zero();
        let d = 4_u16.pow(depth.into());
        let occlusion_rays = (self.render_options.max_occlusion_rays / d).max(1);

        let mut ambient_occlusion = 0;
        for _ in 0..occlusion_rays {
            let direction =
                utils::uniform_sample_cone(&intersection.get_normal(), FRAC_PI_2).into_inner();
            let occlusion_ray = Ray {
                ray_type: RayType::Secondary(depth + 1),
                origin: intersection.get_hit_point() + (direction * BIAS),
                direction,
                refractive_index: 1.0,
            };
            cast_stats.ray_count += 1;
            if !self.shadow_cast(&occlusion_ray, self.render_options.max_occlusion_distance) {
                ambient_occlusion += 1;
            }
        }

        (
            f64::from(ambient_occlusion) / f64::from(occlusion_rays),
            cast_stats,
        )
    }

    #[allow(clippy::option_if_let_else)]
    fn get_color(&self, ray: &Ray) -> (ColorData, CastStats) {
        let mut cast_stats = CastStats::zero();

        if ray.get_depth() >= self.render_options.max_depth {
            return (ColorData::black(), cast_stats);
        }

        cast_stats.ray_count += 1;
        if let Some(mut intersection) = self.raycast(&ray) {
            intersection.compute_data(&ray);

            let material = intersection.object.get_material();
            let (color_data, material_stats) = match material {
                Material::Phong(material) => self.get_color_phong(&ray, &intersection, material),
                Material::Physical(material) => {
                    self.get_color_physical(&ray, &intersection, material)
                }
            };
            cast_stats += material_stats;

            (color_data.clamp(), cast_stats)
        } else {
            (ColorData::black(), cast_stats)
        }
    }

    fn build_camera_rays(&self, x: u32, y: u32) -> Vec<Ray> {
        assert!(x < self.get_width() && y < self.get_height());

        let samples = self.render_options.samples_per_pixel;
        let (width, height) = (f64::from(self.get_width()), f64::from(self.get_height()));
        let aspect = self.get_aspect();
        let fov = self.compute_screen_to_fov();

        let (x, y) = (f64::from(x), f64::from(y));

        let mut ray_pixel_positions = Vec::with_capacity(samples.into());
        ray_pixel_positions.push((x + 0.5, y + 0.5));

        let mut rng = rand::thread_rng();
        for _ in 1..samples {
            let rx: f64 = rng.gen();
            let ry: f64 = rng.gen();
            ray_pixel_positions.push((x + rx, y + ry));
        }

        ray_pixel_positions
            .into_iter()
            .map(|(x, y)| {
                let (x, y) = (
                    utils::remap_value(x, (0.0, width), (-1.0, 1.0)),
                    utils::remap_value(y, (0.0, height), (1.0, -1.0)),
                );

                // Apply fov and scale to aspect ratio
                let (x, y) = if width < height {
                    (x * aspect, y)
                } else {
                    (x, y / aspect)
                };
                let (x, y) = (x * fov, y * fov);

                let direction = Vector3::from([x, y, -1.0]).normalize();
                let direction = (self.camera.camera_to_world * direction.to_homogeneous()).xyz();

                Ray {
                    ray_type: RayType::Primary,
                    origin: self.camera.position,
                    direction,
                    refractive_index: 1.0,
                }
            })
            .collect()
    }

    pub fn screen_raycast(&self, x: u32, y: u32) -> (ColorData, CastStats) {
        let samples = self.render_options.samples_per_pixel;
        let rays = self.build_camera_rays(x, y);

        let (color_data, stats) = if samples == 1 {
            self.get_color(rays.first().unwrap())
        } else {
            let (mut color_data, mut cast_stats) = self.get_color(rays.first().unwrap());

            for ray in &rays[1..] {
                let (data, stats) = self.get_color(ray);
                color_data.color += data.color;
                color_data.ambient_occlusion += data.ambient_occlusion;
                cast_stats += stats;
            }

            let inv_samples = 1.0 / f64::from(samples);
            color_data.color *= inv_samples;
            color_data.ambient_occlusion *= inv_samples;

            (color_data.clamp(), cast_stats)
        };

        (color_data, stats)
    }

    fn post_process_pass(&self, color_data_buffer_lock: &RwLock<Vec<ColorData>>) {
        let width = self.get_width() as usize;
        let blur_radius = self.render_options.occlusion_blur_radius;

        let mut color_data_buffer = color_data_buffer_lock.write().unwrap();
        let ambient_occlusion: Vec<f64> = color_data_buffer
            .iter()
            .map(|color_data| color_data.ambient_occlusion)
            .collect();

        for (color_data, blurred_occlusion) in color_data_buffer
            .iter_mut()
            .zip(utils::repeated_box_blur(&ambient_occlusion, width, blur_radius).into_iter())
        {
            (*color_data).ambient_occlusion = blurred_occlusion;
        }
    }

    fn build_progress_bar(&self) -> ProgressBar {
        let width = u64::from(self.get_width());
        let height = u64::from(self.get_height());

        let progress = ProgressBar::new(width * height);
        progress.set_draw_delta(width * height / 200);
        progress.set_style(
            ProgressStyle::default_bar().template(
                format!(
                    "{} {} {}",
                    "[{elapsed_precise} elapsed] [{eta_precise} left]",
                    "{bar:40}",
                    "{pos}/{len} pixels, {msg} rays",
                )
                .as_str(),
            ),
        );

        progress
    }

    pub fn raytrace_to_image(&self, use_progress: bool) -> (RgbaImage, Duration, CastStats) {
        let width = self.get_width() as usize;
        let height = self.get_height() as usize;

        let mut color_data_buffer: Vec<ColorData> = Vec::new();
        for _ in 0..width * height {
            color_data_buffer.push(ColorData::black());
        }
        let color_data_buffer_lock = RwLock::new(color_data_buffer);
        let mut image_buffer: Vec<u8> = vec![0; width * height * 4];
        let image_buffer_lock = RwLock::new(&mut image_buffer);
        let cast_stats = CastStats::zero();
        let cast_stats_lock = RwLock::new(cast_stats);

        let process_pixel = |&index| {
            let (color_data, stats) =
                self.screen_raycast((index % width) as u32, (index / width) as u32);
            {
                let mut cast_stats = cast_stats_lock.write().unwrap();
                *cast_stats += stats;
            }

            let mut color_data_buffer = color_data_buffer_lock.write().unwrap();
            color_data_buffer[index] = color_data;
        };

        let mut indexes: Vec<usize> = (0..width * height).collect();
        indexes.shuffle(&mut thread_rng());

        let start = Instant::now();
        if use_progress {
            let progress = self.build_progress_bar();

            indexes
                .par_iter()
                .progress_with(progress.clone())
                .inspect(|_| {
                    progress.set_message(&cast_stats_lock.read().unwrap().ray_count.to_string())
                })
                .for_each(process_pixel);

            progress.finish_with_message(&cast_stats_lock.read().unwrap().ray_count.to_string());
        } else {
            indexes.par_iter().for_each(process_pixel);
        }

        self.post_process_pass(&color_data_buffer_lock);

        indexes.iter().for_each(|&index| {
            let color = {
                let color_data_buffer = color_data_buffer_lock.read().unwrap();
                color_data_buffer[index].compute_color_with_gamma_correction()
            };

            let buffer_index = index * 4;
            let mut image_buffer = image_buffer_lock.write().unwrap();
            image_buffer[buffer_index] = (color.x * 255.0) as u8;
            image_buffer[buffer_index + 1] = (color.y * 255.0) as u8;
            image_buffer[buffer_index + 2] = (color.z * 255.0) as u8;
            image_buffer[buffer_index + 3] = 255;
        });

        let duration = start.elapsed();

        let image = RgbaImage::from_raw(width as u32, height as u32, image_buffer)
            .expect("failed to convert buffer");

        let cast_stats = *cast_stats_lock.read().unwrap();
        (image, duration, cast_stats)
    }

    pub fn raytrace_to_buffer(self, use_progress: bool) {
        let width = self.get_width() as usize;
        let height = self.get_height() as usize;

        println!("Rendering to window - press escape to exit.");
        let mut window: Window = Window::new(
            "raytracer",
            width,
            height,
            WindowOptions {
                title: false,
                borderless: true,
                ..WindowOptions::default()
            },
        )
        .unwrap();

        let image_buffer: Vec<u32> = vec![0; width * height];
        let image_buffer_lock = Arc::new(RwLock::new(image_buffer));

        let ray_image_buffer_lock = image_buffer_lock.clone();
        thread::spawn(move || {
            println!("Raytracing...");

            let mut color_data_buffer: Vec<ColorData> = Vec::new();
            for _ in 0..width * height {
                color_data_buffer.push(ColorData::black());
            }
            let color_data_buffer_lock = RwLock::new(color_data_buffer);
            let cast_stats = CastStats::zero();
            let cast_stats_lock = RwLock::new(cast_stats);

            let process_pixel = |&index| {
                let (color_data, stats) =
                    self.screen_raycast((index % width) as u32, (index / width) as u32);
                {
                    let mut cast_stats = cast_stats_lock.write().unwrap();
                    *cast_stats += stats;
                }

                {
                    let mut image_buffer = ray_image_buffer_lock.write().unwrap();
                    image_buffer[index] =
                        utils::to_argb_u32(color_data.compute_color_with_gamma_correction());
                }

                let mut color_data_buffer = color_data_buffer_lock.write().unwrap();
                color_data_buffer[index] = color_data;
            };

            let mut indexes: Vec<usize> = (0..width * height).collect();
            indexes.shuffle(&mut thread_rng());

            if use_progress {
                let progress = self.build_progress_bar();

                indexes
                    .par_iter()
                    .progress_with(progress.clone())
                    .inspect(|_| {
                        progress.set_message(&cast_stats_lock.read().unwrap().ray_count.to_string())
                    })
                    .for_each(process_pixel);

                progress
                    .finish_with_message(&cast_stats_lock.read().unwrap().ray_count.to_string());
            } else {
                indexes.par_iter().for_each(process_pixel);
            }

            self.post_process_pass(&color_data_buffer_lock);

            indexes.iter().for_each(|&index| {
                let color_data_buffer = color_data_buffer_lock.read().unwrap();
                let mut image_buffer = ray_image_buffer_lock.write().unwrap();
                image_buffer[index] = utils::to_argb_u32(color_data_buffer[index].compute_color());
            });
        });

        while window.is_open() && !window.is_key_down(Key::Escape) {
            {
                let image_buffer = image_buffer_lock.read().unwrap();
                window
                    .update_with_buffer(&image_buffer, width, height)
                    .unwrap();
            }

            thread::sleep(Duration::from_millis(100));
        }
    }
}
