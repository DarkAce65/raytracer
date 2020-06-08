use crate::core::{
    self, KdTreeAccelerator, Material, PhongMaterial, PhysicalMaterial, Texture, Transform,
    Transformed,
};
use crate::lights::Light;
use crate::primitives::Object3D;
use crate::ray_intersection::{Intersection, Ray, RayType};
use nalgebra::{clamp, Matrix4, Point3, Unit, Vector2, Vector3, Vector4};
use num_traits::identities::Zero;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::f64::consts::{FRAC_1_PI, FRAC_PI_2};
use std::fmt;
use std::path::Path;

const GAMMA: f64 = 2.2;
const BIAS: f64 = 1e-10;

#[derive(Debug)]
pub struct Camera {
    fov: f64,
    position: Point3<f64>,
    camera_to_world: Matrix4<f64>,
}

impl Camera {
    pub fn new(fov: f64, eye: Point3<f64>, target: Point3<f64>, up: Unit<Vector3<f64>>) -> Self {
        Self {
            fov,
            position: eye,
            camera_to_world: Matrix4::look_at_rh(&eye, &target, &up).transpose(),
        }
    }

    pub fn default_fov() -> f64 {
        65.0
    }
    pub fn default_position() -> Point3<f64> {
        Point3::from([0.0, 0.0, 1.0])
    }
    pub fn default_target() -> Point3<f64> {
        Point3::origin()
    }
    pub fn default_up() -> Unit<Vector3<f64>> {
        Vector3::y_axis()
    }
}

impl<'de> Deserialize<'de> for Camera {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Fov,
            Position,
            Target,
            Up,
        }

        struct CameraVisitor;

        impl<'de> Visitor<'de> for CameraVisitor {
            type Value = Camera;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Camera")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Camera, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut fov = None;
                let mut position = None;
                let mut target = None;
                let mut up = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Fov => {
                            if fov.is_some() {
                                return Err(de::Error::duplicate_field("fov"));
                            }
                            fov = Some(map.next_value()?);
                        }
                        Field::Position => {
                            if position.is_some() {
                                return Err(de::Error::duplicate_field("position"));
                            }
                            position = Some(map.next_value()?);
                        }
                        Field::Target => {
                            if target.is_some() {
                                return Err(de::Error::duplicate_field("target"));
                            }
                            target = Some(map.next_value()?);
                        }
                        Field::Up => {
                            if up.is_some() {
                                return Err(de::Error::duplicate_field("up"));
                            }
                            up = Some(map.next_value()?);
                        }
                    }
                }

                let fov = fov.unwrap_or_else(Camera::default_fov);
                let position = position.ok_or_else(|| de::Error::missing_field("position"))?;
                let target = target.unwrap_or_else(Camera::default_target);
                let up = up.unwrap_or_else(Camera::default_up);

                Ok(Camera::new(fov, position, target, up))
            }
        }

        deserializer.deserialize_map(CameraVisitor)
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RenderOptions {
    width: u32,
    height: u32,
    max_depth: u8,
    max_reflected_rays: u8,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            max_depth: 3,
            max_reflected_rays: 8,
            width: 100,
            height: 100,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Scene {
    #[serde(flatten)]
    render_options: RenderOptions,
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
            camera: Camera::new(
                Camera::default_fov(),
                Camera::default_position(),
                Camera::default_target(),
                Camera::default_up(),
            ),
            lights: Vec::new(),
            objects: Vec::new(),

            textures: HashMap::new(),
        }
    }
}

impl Scene {
    pub fn load_assets(&mut self, asset_base: &Path) {
        for object in &mut self.objects {
            Object3D::load_assets(object, asset_base, &mut self.textures);
        }
    }

    pub fn build_raytracing_scene(self) -> RaytracingScene {
        RaytracingScene::new(self)
    }
}

#[derive(Debug)]
pub struct RaytracingScene {
    render_options: RenderOptions,
    camera: Camera,
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
            camera: scene.camera,
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

    fn raycast(&self, ray: &Ray) -> Option<Intersection> {
        self.object_tree.raycast(ray)
    }

    fn shadow_cast(&self, ray: &Ray, max_distance: f64) -> bool {
        self.object_tree.shadow_cast(ray, max_distance - BIAS)
    }

    fn get_color_phong(
        &self,
        ray: Ray,
        hit_point: Point3<f64>,
        normal: Unit<Vector3<f64>>,
        uv: Vector2<f64>,
        material: &PhongMaterial,
    ) -> (Vector4<f64>, u64) {
        let mut ray_count = 0;
        let depth = ray.get_depth();

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
            let (color, r) = self.get_color(reflection_ray);
            ray_count += r;
            color.xyz().component_mul(&material_color)
        } else {
            Vector3::zero()
        };

        let mut ambient_light = Vector3::zero();
        let mut irradiance = Vector3::zero();
        if material.reflectivity < 1.0 {
            for light in self.lights.iter() {
                match light {
                    Light::Ambient(light) => {
                        ambient_light += light.color.component_mul(&material_color);
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
                                irradiance += light.color.component_mul(&material_color) * n_dot_l;

                                let half_vec = Unit::new_normalize(light_dir - ray.direction);
                                let n_dot_h = normal.dot(&half_vec);
                                if n_dot_h > 0.0 {
                                    irradiance += light.color.component_mul(&material.specular)
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
        ray: Ray,
        hit_point: Point3<f64>,
        normal: Unit<Vector3<f64>>,
        uv: Vector2<f64>,
        material: &PhysicalMaterial,
    ) -> (Vector4<f64>, u64) {
        let mut ray_count = 0;
        let depth = ray.get_depth();

        let material_color = material.get_color(uv, &self.textures);

        let view_dir = Unit::new_normalize(-ray.direction);
        let n_dot_v = normal.dot(&view_dir).max(0.0);

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
                let (color, r) = self.get_color(refraction_ray);
                ray_count += r;
                refraction += color.xyz().component_mul(&material_color);
            }
        }

        let mut reflection: Vector3<f64> = Vector3::zero();
        if self.render_options.max_reflected_rays > 0 {
            let d = 0.5f64.powi(depth as i32);
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
                    let (color, r) = self.get_color(reflection_ray);
                    ray_count += r;
                    reflection += FRAC_PI_2 * color.xyz().component_mul(&f);
                }
                reflection /= reflected_rays as f64;
            }
        }

        let mut ambient_light = Vector3::zero();
        let mut irradiance = Vector3::zero();
        let diffuse = material_color * FRAC_1_PI;
        for light in self.lights.iter() {
            match light {
                Light::Ambient(light) => {
                    ambient_light += light.color.component_mul(&material_color);
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

                            let radiance = light.color * n_dot_l;

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

    fn get_color(&self, ray: Ray) -> (Vector4<f64>, u64) {
        let mut ray_count = 0;

        if ray.get_depth() >= self.render_options.max_depth {
            return (Vector4::zero(), ray_count);
        }

        ray_count += 1;
        if let Some(mut intersection) = self.raycast(&ray) {
            intersection.compute_data(&ray);
            let hit_point = intersection.get_hit_point();
            let normal = intersection.get_normal();
            let uv = intersection.get_uv();
            let material = intersection.object.get_material();

            let (color, r) = match material {
                Material::Phong(material) => {
                    self.get_color_phong(ray, hit_point, normal, uv, material)
                }
                Material::Physical(material) => {
                    self.get_color_physical(ray, hit_point, normal, uv, material)
                }
            };

            (color.map(|c| clamp(c, 0.0, 1.0)), ray_count + r)
        } else {
            (Vector4::zero(), ray_count)
        }
    }

    pub fn screen_raycast(&self, x: f64, y: f64) -> (Vector4<f64>, u64) {
        let (width, height) = (self.get_width() as f64, self.get_height() as f64);
        assert!(0.0 <= x && x < width);
        assert!(0.0 <= y && y < height);

        let aspect = width / height;
        let fov = (self.camera.fov.to_radians() / 2.0).tan();

        let (x, y) = ((x + 0.5) / width, (y + 0.5) / height);
        let (x, y) = (x * 2.0 - 1.0, 1.0 - y * 2.0);
        let (x, y) = if width < height {
            (x * aspect, y)
        } else {
            (x, y / aspect)
        };
        let (x, y) = (x * fov, y * fov);

        let direction = Vector3::from([x, y, -1.0]).normalize();
        let direction = (self.camera.camera_to_world * direction.to_homogeneous()).xyz();

        let ray = Ray {
            ray_type: RayType::Primary,
            origin: self.camera.position,
            direction,
            refractive_index: 1.0,
        };

        let (color, ray_count) = self.get_color(ray);
        (color.map(|c| c.powf(1.0 / GAMMA)), ray_count)
    }
}
