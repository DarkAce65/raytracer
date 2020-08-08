use crate::core::{
    self, remap_value, KdTreeAccelerator, Material, PhongMaterial, PhysicalMaterial, Texture,
    Transform, Transformed,
};
use crate::lights::Light;
use crate::primitives::Object3D;
use crate::ray_intersection::{Intersection, Ray, RayType};
use image::RgbaImage;
use indicatif::{ParallelProgressIterator, ProgressBar};
use nalgebra::{clamp, Matrix4, Point3, Unit, Vector3, Vector4};
use num_traits::identities::Zero;
use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::f64::consts::{FRAC_1_PI, FRAC_PI_2};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::{Duration, Instant};

const GAMMA: f64 = 2.2;
const BIAS: f64 = 1e-10;

fn to_argb_u32(rgba: Vector4<f64>) -> u32 {
    let (r, g, b, a) = (
        (rgba.x * 255.0) as u32,
        (rgba.y * 255.0) as u32,
        (rgba.z * 255.0) as u32,
        (rgba.w * 255.0) as u32,
    );
    a << 24 | r << 16 | g << 8 | b
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Camera {
    pub fov: f64,
    pub position: Point3<f64>,
    pub target: Point3<f64>,
    pub up: Unit<Vector3<f64>>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 65.0,
            position: Point3::from([0.0, 0.0, 1.0]),
            target: Point3::origin(),
            up: Vector3::y_axis(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: u8,
    pub max_depth: u8,
    pub max_reflected_rays: u8,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            samples_per_pixel: 4,
            max_depth: 3,
            max_reflected_rays: 32,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Scene {
    #[serde(flatten)]
    render_options: RenderOptions,
    loaded: bool,
    camera: Camera,
    lights: Vec<Light>,
    objects: Vec<Object3D>,

    #[serde(skip)]
    textures: HashMap<String, Texture>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            render_options: RenderOptions::default(),
            loaded: false,
            camera: Camera::default(),
            lights: Vec::new(),
            objects: Vec::new(),

            textures: HashMap::new(),
        }
    }
}

impl Scene {
    pub fn new(render_options: RenderOptions, camera: Camera) -> Self {
        Self {
            render_options,
            camera,
            ..Scene::default()
        }
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light)
    }

    pub fn add_object(&mut self, object: Object3D) {
        if self.loaded {
            panic!("objects cannot be added after scene assets have loaded")
        }

        self.objects.push(object)
    }

    pub fn load_assets(&mut self, asset_base: &Path) {
        if self.loaded {
            panic!("assets are already loaded for scene")
        }

        for object in &mut self.objects {
            Object3D::load_assets(object, asset_base, &mut self.textures);
        }
        self.loaded = true;
    }

    pub fn build_raytracing_scene(self) -> RaytracingScene {
        RaytracingScene::new(self)
    }
}

#[derive(Debug)]
struct RaytracingCamera {
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
    render_options: RenderOptions,
    camera: RaytracingCamera,
    lights: Vec<Light>,
    textures: HashMap<String, Texture>,
    object_tree: KdTreeAccelerator,
}

impl RaytracingScene {
    fn new(scene: Scene) -> Self {
        let root_transform = Transform::default();
        let mut objects = Vec::new();
        for object in scene.objects {
            objects.append(&mut object.flatten_to_world(&root_transform));
        }
        let object_tree = KdTreeAccelerator::new(objects);

        Self {
            render_options: scene.render_options,
            camera: scene.camera.into(),
            lights: scene.lights,
            textures: scene.textures,
            object_tree,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.render_options.width
    }

    pub fn get_height(&self) -> u32 {
        self.render_options.height
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
    ) -> (Vector4<f64>, u64) {
        let mut ray_count = 0;
        let depth = ray.get_depth();
        let hit_point = intersection.get_hit_point();

        let normal = intersection.get_normal();

        let uv = intersection.get_uv();
        let material_color = material.get_color(uv, &self.textures);
        let emissive_light = material.emissive;

        let reflection = if material.reflectivity > 0.0 {
            let reflection_dir = core::reflect(&ray.direction, &normal).into_inner();
            let reflection_ray = Ray {
                ray_type: RayType::Secondary(depth + 1),
                origin: hit_point + (reflection_dir * BIAS),
                direction: reflection_dir,
                refractive_index: 1.0,
            };
            let (color, r) = self.get_color(&reflection_ray);
            ray_count += r;
            color.xyz().component_mul(&material_color)
        } else {
            Vector3::zero()
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

                            ray_count += 1;
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

        let color = emissive_light
            + ambient_light
            + material.reflectivity * reflection
            + (1.0 - material.reflectivity) * irradiance;

        (color.insert_row(3, 1.0), ray_count)
    }

    fn get_color_physical(
        &self,
        ray: &Ray,
        intersection: &Intersection,
        material: &PhysicalMaterial,
    ) -> (Vector4<f64>, u64) {
        let mut ray_count = 0;
        let depth = ray.get_depth();
        let hit_point = intersection.get_hit_point();

        let normal = intersection.get_normal();
        let view_dir = Unit::new_normalize(-ray.direction);
        let n_dot_v = normal.dot(&view_dir).max(0.0);

        let uv = intersection.get_uv();
        let material_color = material.get_color(uv, &self.textures);

        let roughness = material.roughness.max(0.04);
        let base_reflectivity = Vector3::repeat(0.04).lerp(&material_color, material.metalness);
        let f = core::fresnel(n_dot_v, base_reflectivity);
        let k_s = f;
        let k_d = (Vector3::repeat(1.0) - k_s) * (1.0 - material.metalness);

        let emissive_light = material.emissive;

        let mut refraction: Vector3<f64> = Vector3::zero();
        if material.opacity < 1.0 {
            let eta = ray.refractive_index / material.refractive_index;
            if let Some(refraction_dir) = core::refract(&ray.direction, &normal, eta) {
                let refraction_dir = refraction_dir.into_inner();
                let refraction_ray = Ray {
                    ray_type: RayType::Secondary(depth + 1),
                    origin: hit_point + (refraction_dir * BIAS),
                    direction: refraction_dir,
                    refractive_index: material.refractive_index,
                };
                let (color, r) = self.get_color(&refraction_ray);
                ray_count += r;
                refraction += color.xyz().component_mul(&material_color);
            }
        }

        let mut reflection: Vector3<f64> = Vector3::zero();
        if self.render_options.max_reflected_rays > 0 {
            let d = 0.125_f64.powi(depth as i32);
            let reflected_rays = (self.render_options.max_reflected_rays as f64 * d) as u8;
            if reflected_rays > 0 {
                let max_angle = (FRAC_PI_2 * material.roughness).cos();
                let reflection_dir = core::reflect(&ray.direction, &normal);
                for _ in 0..reflected_rays {
                    let direction =
                        core::uniform_sample_cone(&reflection_dir, max_angle).into_inner();
                    let reflection_ray = Ray {
                        ray_type: RayType::Secondary(depth + 1),
                        origin: hit_point + (direction * BIAS),
                        direction,
                        refractive_index: 1.0,
                    };
                    let (color, r) = self.get_color(&reflection_ray);
                    ray_count += r;
                    reflection += FRAC_PI_2 * color.xyz().component_mul(&f);
                }
                reflection /= reflected_rays as f64;
            }
        }

        let mut ambient_light = Vector3::zero();
        let mut irradiance = Vector3::zero();
        let diffuse = material_color * FRAC_1_PI;
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

                        ray_count += 1;
                        if !self.shadow_cast(&shadow_ray, light_distance) {
                            let half_vec = Unit::new_normalize(light_dir - ray.direction);
                            let n_dot_h = normal.dot(&half_vec).max(0.0);

                            let light_color = light.get_color(light_distance);
                            let radiance = light_color * n_dot_l;

                            let ndf = core::ndf(n_dot_h, roughness);
                            let g = core::geometry_function(n_dot_v, n_dot_l, roughness);

                            let specular = if n_dot_v == 0.0 {
                                Vector3::zero()
                            } else {
                                ndf * g * f / (4.0 * n_dot_v * n_dot_l)
                            };

                            irradiance += (k_d.component_mul(&diffuse) + specular)
                                .component_mul(&radiance)
                                * n_dot_l;
                        }
                    }
                }
            };
        }

        let color = (1.0 - material.opacity) * k_s.component_mul(&refraction)
            + material.opacity * (emissive_light + ambient_light + reflection + irradiance);

        (color.insert_row(3, 1.0), ray_count)
    }

    fn get_color(&self, ray: &Ray) -> (Vector4<f64>, u64) {
        let mut ray_count = 0;

        if ray.get_depth() >= self.render_options.max_depth {
            return (Vector4::zero(), ray_count);
        }

        ray_count += 1;
        if let Some(mut intersection) = self.raycast(&ray) {
            intersection.compute_data(&ray);
            let material = intersection.object.get_material();

            let (color, r) = match material {
                Material::Phong(material) => self.get_color_phong(&ray, &intersection, material),
                Material::Physical(material) => {
                    self.get_color_physical(&ray, &intersection, material)
                }
            };

            (color.map(|c| clamp(c, 0.0, 1.0)), ray_count + r)
        } else {
            (Vector4::zero(), ray_count)
        }
    }

    fn build_camera_rays(&self, x: u32, y: u32, samples: u8) -> Vec<Ray> {
        assert!(x < self.get_width() && y < self.get_height());
        assert!(samples >= 1);

        let (x, y) = (x as f64, y as f64);
        let (width, height) = (
            (self.get_width() - 1) as f64,
            (self.get_height() - 1) as f64,
        );
        let aspect = width / height;
        let fov = (self.camera.fov.to_radians() / 2.0).tan();

        let mut ray_pixel_positions = Vec::with_capacity(samples.into());
        ray_pixel_positions.push((x + 0.5, y + 0.5));

        let mut rng = rand::thread_rng();
        for _ in 0..(samples - 1) {
            let rx: f64 = rng.gen();
            let ry: f64 = rng.gen();
            ray_pixel_positions.push((x + rx, y + ry));
        }

        ray_pixel_positions
            .into_iter()
            .map(|(x, y)| {
                let (x, y) = (
                    remap_value(x, (0.0, width), (-1.0, 1.0)),
                    remap_value(y, (0.0, height), (1.0, -1.0)),
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

    pub fn screen_raycast(&self, x: u32, y: u32) -> (Vector4<f64>, u64) {
        let samples = self.render_options.samples_per_pixel;
        let rays = self.build_camera_rays(x, y, samples);

        let (color, ray_count) = if samples == 1 {
            self.get_color(rays.first().unwrap())
        } else {
            let mut color = Vector4::zero();
            let mut ray_count = 0;
            for ray in &rays {
                let (c, r) = self.get_color(ray);
                color += c;
                ray_count += r;
            }

            (color / samples as f64, ray_count)
        };

        (color.map(|c| c.powf(1.0 / GAMMA)), ray_count)
    }

    pub fn raytrace_to_image(&self, progress: Option<ProgressBar>) -> (RgbaImage, Duration, u64) {
        let width = self.get_width();
        let height = self.get_height();

        let mut image_buffer: Vec<u8> = vec![0; (width * height * 4) as usize];
        let buffer_mutex = Arc::new(Mutex::new(&mut image_buffer));
        let rays = AtomicU64::new(0);

        let process_pixel = |index| {
            let (color, r) = self.screen_raycast(index % width, index / width);
            rays.fetch_add(r, Ordering::SeqCst);

            let index = (index * 4) as usize;
            let mut buffer = buffer_mutex.lock().unwrap();
            buffer[index] = (color.x * 255.0) as u8;
            buffer[index + 1] = (color.y * 255.0) as u8;
            buffer[index + 2] = (color.z * 255.0) as u8;
            buffer[index + 3] = (color.w * 255.0) as u8;
        };

        let mut indexes: Vec<u32> = (0..width * height).collect();
        indexes.shuffle(&mut thread_rng());

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
        let duration = start.elapsed();
        let image =
            RgbaImage::from_raw(width, height, image_buffer).expect("failed to convert buffer");

        (image, duration, rays.load(Ordering::SeqCst))
    }

    pub fn raytrace_to_buffer(
        self,
        buffer_mutex: &Arc<Mutex<Vec<u32>>>,
        progress: Option<ProgressBar>,
    ) {
        let width = self.get_width();
        let height = self.get_height();

        assert!(buffer_mutex.lock().unwrap().len() == (width * height) as usize);

        let buffer_mutex = Arc::clone(buffer_mutex);
        spawn(move || {
            let rays = AtomicU64::new(0);

            let process_pixel = |index| {
                let (color, r) = self.screen_raycast(index % width, index / width);
                rays.fetch_add(r, Ordering::SeqCst);

                let mut buffer = buffer_mutex.lock().unwrap();
                buffer[index as usize] = to_argb_u32(color);
            };

            let mut indexes: Vec<u32> = (0..width * height).collect();
            indexes.shuffle(&mut thread_rng());

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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::{Material, PhongMaterial};
    use crate::lights::{AmbientLight, PointLight};
    use crate::primitives::Cube;
    use serde_json::json;

    #[allow(clippy::shadow_unrelated)]
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
        assert_eq!(to_argb_u32(Vector4::from([1.0, 0.0, 1.0, 1.0])), color);
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_an_empty_scene_json() {
        let scene_json = json!({});
        let scene: Result<Scene, serde_json::error::Error> = serde_json::from_value(scene_json);
        assert!(scene.is_ok(), "failed to deserialize scene");

        scene.unwrap().build_raytracing_scene();
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_a_scene_json() {
        let scene_json = json!({
          "max_depth": 5,
          "width": 200,
          "height": 200,
          "camera": { "position": [2, 5, 15], "target": [-1, 0, 0] },
          "lights": [
            { "type": "ambient", "color": [0.01, 0.01, 0.01] },
            {
              "type": "point",
              "transform": [{ "translate": [-8, 3, 0] }],
              "color": [0.5, 0.5, 0.5]
            }
          ],
          "objects": [
            {
              "type": "cube",
              "size": 1,
              "transform": [{ "rotate": [[0, 1, 0], 30] }, { "translate": [0, 2, 0] }],
              "material": { "type": "phong", "color": [1, 0.1, 0.1] }
            }
          ]
        });

        let scene: Result<Scene, serde_json::error::Error> = serde_json::from_value(scene_json);
        assert!(scene.is_ok(), "failed to deserialize scene");

        scene.unwrap().build_raytracing_scene();
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_an_empty_scene() {
        let scene = Scene::new(RenderOptions::default(), Camera::default());
        scene.build_raytracing_scene();
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_a_scene() {
        let mut scene = Scene::new(
            RenderOptions {
                width: 200,
                height: 200,
                max_depth: 5,
                ..RenderOptions::default()
            },
            Camera::default(),
        );

        scene.add_light(Light::Ambient(AmbientLight::new(Vector3::from([
            0.01, 0.01, 0.01,
        ]))));
        scene.add_light(Light::Point(Box::new(PointLight::new(
            Vector3::from([0.5, 0.5, 0.5]),
            1.0,
            Transform::identity().translate(Vector3::from([-8.0, 3.0, 0.0])),
        ))));

        scene.add_object(Object3D::Cube(Box::new(Cube::new(
            1.0,
            Transform::identity()
                .rotate(Vector3::y_axis(), 30.0)
                .translate(Vector3::from([0.0, 2.0, 0.0])),
            Material::Phong(PhongMaterial {
                color: Vector3::from([1.0, 0.1, 0.1]),
                ..PhongMaterial::default()
            }),
        ))));

        scene.build_raytracing_scene();
    }
}
