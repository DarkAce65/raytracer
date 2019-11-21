use crate::core::{Intersection, Ray};
use crate::lights::{Light, LightType};
use crate::primitives::{MaterialSide, Primitive};
use nalgebra::{Matrix4, Point3, Unit, Vector3, Vector4};
use num_traits::identities::Zero;
use std::cmp::Ordering::Equal;

#[derive(Debug)]
pub struct Camera {
    fov: f64,
    position: Point3<f64>,
    camera_to_world: Matrix4<f64>,
}

impl Camera {
    pub fn from(fov: f64, eye: Point3<f64>, target: Point3<f64>, up: Unit<Vector3<f64>>) -> Self {
        Self {
            fov,
            position: eye,
            camera_to_world: Matrix4::look_at_rh(&eye, &target, &up).transpose(),
        }
    }
}

#[derive(Debug)]
pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub camera: Camera,
    pub lights: Vec<Box<dyn Light>>,
    pub objects: Vec<Box<dyn Primitive>>,
}

impl Scene {
    fn raycast(&self, ray: &Ray) -> Option<Intersection> {
        self.objects
            .iter()
            .filter_map(|object| object.intersect(&ray))
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal))
    }

    fn get_color(&self, ray: Ray) -> Vector4<f64> {
        if let Some(intersection) = self.raycast(&ray) {
            let hit_point = ray.origin + ray.direction.into_inner() * intersection.distance;
            let material = intersection.object.material();
            let normal = intersection.object.surface_normal(&hit_point);
            let normal = match material.side {
                MaterialSide::Front => normal,
                MaterialSide::Back => -normal,
            };

            let mut color = Vector3::zero();

            for light in self.lights.iter() {
                color += match light.get_type() {
                    LightType::Ambient => material.color.component_mul(&light.get_color()),
                    LightType::Point => {
                        let mut light_color = Vector3::zero();

                        let light_dir = light.position() - hit_point;
                        let light_distance = light_dir.magnitude();
                        let light_dir = Unit::new_normalize(light_dir);

                        let n_dot_l = normal.dot(&light_dir);
                        if n_dot_l > 0.0 {
                            let shadow_ray = Ray {
                                origin: hit_point + (normal.into_inner() * 1e-10),
                                direction: light_dir,
                            };

                            let shadow_intersection = self.raycast(&shadow_ray);
                            if shadow_intersection.is_none()
                                || shadow_intersection.unwrap().distance > light_distance
                            {
                                light_color +=
                                    material.color.component_mul(&light.get_color()) * n_dot_l;

                                let half_vec = (light_dir.into_inner()
                                    - ray.direction.into_inner())
                                .normalize();
                                let n_dot_h = normal.dot(&half_vec);
                                if n_dot_h > 0.0 {
                                    light_color +=
                                        material.specular.component_mul(&light.get_color())
                                            * n_dot_h.powf(material.shininess);
                                }
                            }
                        }

                        light_color
                    }
                };
            }

            color.insert_row(3, 1.0)
        } else {
            Vector4::zero()
        }
    }

    pub fn screen_raycast(&self, index: u32) -> Vector4<f64> {
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

        let direction = Vector4::from([x, y, -1.0, 1.0]).normalize();
        let direction = Unit::new_normalize((self.camera.camera_to_world * direction).xyz());

        let ray = Ray {
            origin: self.camera.position,
            direction,
        };

        self.get_color(ray)
    }
}
