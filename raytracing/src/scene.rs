use crate::core::{self, uniform_sample_cone, Intersection, Ray};
use crate::core::{Material, MaterialSide, PhongMaterial, PhysicalMaterial};
use crate::lights::{Light, LightType};
use crate::primitives::Primitive;
use nalgebra::{clamp, Matrix4, Point3, Unit, Vector3, Vector4};
use num_traits::identities::Zero;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Ordering::Equal;
use std::f64::consts::{FRAC_1_PI, FRAC_2_PI};
use std::fmt;

const BIAS: f64 = 1e-10;
const MAX_DEPTH: u8 = 2;
const REFLECTED_RAYS: u8 = 16;

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub camera: Camera,
    pub lights: Vec<Box<dyn Light>>,
    pub objects: Vec<Box<dyn Primitive>>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            camera: Camera::new(
                Camera::default_fov(),
                Camera::default_position(),
                Camera::default_target(),
                Camera::default_up(),
            ),
            lights: Vec::new(),
            objects: Vec::new(),
        }
    }
}

impl Scene {
    fn raycast(&self, ray: &Ray) -> Option<Intersection> {
        self.objects
            .iter()
            .filter_map(|object| object.intersect(&ray))
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal))
    }

    fn get_color_phong(
        &self,
        ray: Ray,
        depth: u8,
        hit_point: Point3<f64>,
        normal: Unit<Vector3<f64>>,
        material: PhongMaterial,
    ) -> (Vector4<f64>, u64) {
        let mut ray_count = 0;

        let emissive_light = material.emissive;

        let reflection_dir =
            (ray.direction - 2.0 * ray.direction.dot(&normal) * normal.into_inner()).normalize();
        let reflection_ray = Ray {
            origin: hit_point + (reflection_dir * BIAS),
            direction: reflection_dir,
        };
        let (color, r) = self.get_color(reflection_ray, depth - 1);
        ray_count += r;
        let reflection =
            material.reflectivity * FRAC_2_PI * color.xyz().component_mul(&material.color);

        let mut ambient_light = Vector3::zero();
        let mut irradiance = Vector3::zero();
        for light in self.lights.iter() {
            match light.get_type() {
                LightType::Ambient => {
                    ambient_light += light.get_color().component_mul(&material.color);
                }
                LightType::Point => {
                    let light_dir = light.transform().matrix() * Point3::origin() - hit_point;
                    let light_distance = light_dir.magnitude();
                    let light_dir = light_dir.normalize();

                    let n_dot_l = normal.dot(&light_dir);
                    if n_dot_l > 0.0 {
                        let shadow_ray = Ray {
                            origin: hit_point + (light_dir * BIAS),
                            direction: light_dir,
                        };

                        ray_count += 1;
                        let shadow_intersection = self.raycast(&shadow_ray);
                        if shadow_intersection.is_none()
                            || shadow_intersection.unwrap().distance > light_distance
                        {
                            irradiance +=
                                light.get_color().component_mul(&material.color) * n_dot_l;

                            let half_vec = Unit::new_normalize(light_dir - ray.direction);
                            let n_dot_h = normal.dot(&half_vec);
                            if n_dot_h > 0.0 {
                                irradiance += material.specular.component_mul(&light.get_color())
                                    * n_dot_h.powf(material.shininess);
                            }
                        }
                    }
                }
            };
        }

        let color = emissive_light + ambient_light + reflection + irradiance;

        (color.insert_row(3, 1.0), ray_count)
    }

    fn get_color_physical(
        &self,
        ray: Ray,
        depth: u8,
        hit_point: Point3<f64>,
        normal: Unit<Vector3<f64>>,
        material: PhysicalMaterial,
    ) -> (Vector4<f64>, u64) {
        let mut ray_count = 0;

        let view_dir = Unit::new_normalize(-ray.direction);
        let n_dot_v = normal.dot(&view_dir).max(0.0);

        let base_reflectivity = Vector3::repeat(0.04).lerp(&material.color, material.metalness);
        let roughness = material.roughness.max(0.04);

        let emissive_light = material.emissive;

        let mut reflection: Vector3<f64> = Vector3::zero();

        let max_angle = (FRAC_2_PI * roughness).cos();
        let reflection_dir = Unit::new_normalize(
            ray.direction - 2.0 * ray.direction.dot(&normal) * normal.into_inner(),
        );
        if REFLECTED_RAYS > 0 {
            for _ in 0..REFLECTED_RAYS {
                let direction = uniform_sample_cone(&reflection_dir, max_angle).into_inner();
                let reflection_ray = Ray {
                    origin: hit_point + (direction * BIAS),
                    direction,
                };
                let (color, r) = self.get_color(reflection_ray, depth - 1);
                ray_count += r;
                reflection += FRAC_2_PI * color.xyz();
            }
            reflection /= REFLECTED_RAYS as f64;
        }

        let diffuse = material.color * FRAC_1_PI;

        let mut ambient_light = Vector3::zero();
        let mut irradiance = Vector3::zero();
        for light in self.lights.iter() {
            match light.get_type() {
                LightType::Ambient => {
                    ambient_light += light.get_color().component_mul(&material.color);
                }
                LightType::Point => {
                    let light_dir = light.transform().matrix() * Point3::origin() - hit_point;
                    let light_distance = light_dir.magnitude();
                    let light_dir = light_dir.normalize();

                    let n_dot_l = normal.dot(&light_dir);
                    if n_dot_l > 0.0 {
                        let shadow_ray = Ray {
                            origin: hit_point + (light_dir * BIAS),
                            direction: light_dir,
                        };

                        ray_count += 1;
                        let shadow_intersection = self.raycast(&shadow_ray);
                        if shadow_intersection.is_none()
                            || shadow_intersection.unwrap().distance > light_distance
                        {
                            let half_vec = Unit::new_normalize(light_dir - ray.direction);
                            let n_dot_h = normal.dot(&half_vec).max(0.0);
                            let attenuation = 1.0 / light_distance * light_distance;

                            let radiance = light.get_color() * attenuation * n_dot_l;

                            let ndf = core::ndf(n_dot_h, roughness);
                            let g = core::geometry_function(n_dot_v, n_dot_l, roughness);
                            let f = core::fresnel(n_dot_v, base_reflectivity);

                            let specular = if n_dot_v == 0.0 {
                                Vector3::zero()
                            } else {
                                ndf * g * f / (4.0 * n_dot_v * n_dot_l)
                            };

                            let k_s = f;
                            let k_d = (Vector3::repeat(1.0) - k_s) * (1.0 - material.metalness);

                            irradiance += (k_d.component_mul(&diffuse) + specular)
                                .component_mul(&radiance)
                                * n_dot_l;
                        }
                    }
                }
            };
        }

        let color = emissive_light + ambient_light + reflection + irradiance;
        let color = color
            .map(|c| (c / (c + 1.0)).powf(1.0 / 2.2))
            .map(|c| clamp(c, 0.0, 1.0));

        (color.insert_row(3, 1.0), ray_count)
    }

    fn get_color(&self, ray: Ray, depth: u8) -> (Vector4<f64>, u64) {
        let mut ray_count = 0;

        if depth == 0 {
            return (Vector4::zero(), ray_count);
        }

        ray_count += 1;
        if let Some(intersection) = self.raycast(&ray) {
            let hit_point = ray.origin + ray.direction * intersection.distance;
            let material = intersection.object.material();
            let normal = intersection
                .object
                .surface_normal(&(intersection.object.transform().inverse() * hit_point));
            let normal = Unit::new_normalize(
                intersection.object.transform().inverse_transpose() * normal.into_inner(),
            );
            let normal = match material.side() {
                MaterialSide::Front => normal,
                MaterialSide::Back => -normal,
            };

            match material {
                Material::Phong(material) => {
                    let (color, r) = self.get_color_phong(ray, depth, hit_point, normal, material);

                    (color, ray_count + r)
                }
                Material::Physical(material) => {
                    let (color, r) =
                        self.get_color_physical(ray, depth, hit_point, normal, material);

                    (color, ray_count + r)
                }
            }
        } else {
            (Vector4::zero(), ray_count)
        }
    }

    pub fn screen_raycast(&self, index: u32) -> (Vector4<f64>, u64) {
        assert!(index < self.width * self.height);

        let (width, height) = (self.width as f64, self.height as f64);
        let aspect = width / height;
        let fov = (self.camera.fov.to_radians() / 2.0).tan();

        let (x, y) = ((index % self.width) as f64, (index / self.width) as f64);
        let (x, y) = ((x + 0.5) / width, (y + 0.5) / height);
        let (x, y) = (x * 2.0 - 1.0, 1.0 - y * 2.0);
        let (x, y) = if self.width < self.height {
            (x * aspect, y)
        } else {
            (x, y / aspect)
        };
        let (x, y) = (x * fov, y * fov);

        let direction = Vector3::from([x, y, -1.0]).normalize();
        let direction = (self.camera.camera_to_world * direction.to_homogeneous()).xyz();

        let ray = Ray {
            origin: self.camera.position,
            direction,
        };

        self.get_color(ray, MAX_DEPTH)
    }
}
