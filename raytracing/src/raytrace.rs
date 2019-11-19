use crate::lights::Light;
use crate::primitives::{Intersection, Primitive};
use nalgebra::{Vector3, Vector4};
use num_traits::identities::Zero;
use std::cmp::Ordering::Equal;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

pub trait Object3D {
    fn position(&self) -> Vector3<f32>;
    fn scale(&self) -> Vector3<f32>;
    fn rotation(&self) -> Vector3<f32>;
}

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f32,
    pub lights: Vec<Box<dyn Light>>,
    pub objects: Vec<Box<dyn Primitive>>,
}

impl Scene {
    fn index_to_dir(&self, index: u32) -> Vector3<f32> {
        assert!(index < self.width * self.height);

        let (w, h) = (self.width as f32, self.height as f32);
        let (x, y) = ((index % self.width) as f32, (index / self.width) as f32);

        let aspect = w / h;
        let fov = (self.fov.to_radians() / 2.0).tan();
        let x = (((x + 0.5) / w) * 2.0 - 1.0) * fov;
        let y = (1.0 - ((y + 0.5) / h) * 2.0) * fov;

        if self.width < self.height {
            Vector3::from([x * aspect, y, -1.0]).normalize()
        } else {
            Vector3::from([x, y / aspect, -1.0]).normalize()
        }
    }

    fn raycast(&self, ray: &Ray) -> Option<Intersection> {
        self.objects
            .iter()
            .filter_map(|object| object.intersect(&ray))
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal))
    }

    fn get_color(&self, ray: &Ray) -> Vector4<f32> {
        if let Some(intersection) = self.raycast(ray) {
            let hit_point = ray.origin + ray.direction * intersection.distance;
            let normal = intersection.object.surface_normal(&hit_point);

            let mut color = Vector3::zero();
            for light in self.lights.iter() {
                let light_dir = (light.position() - hit_point).normalize();
                let shadow_ray = Ray {
                    origin: hit_point + (normal * 5e-3),
                    direction: light_dir,
                };
                if self.raycast(&shadow_ray).is_none() {
                    color += intersection.object.color().xyz() * normal.dot(&light_dir);
                }
            }

            color.insert_row(3, intersection.object.color().w)
        } else {
            Vector4::zero()
        }
    }

    pub fn screen_raycast(&self, index: u32) -> Vector4<f32> {
        let ray = Ray {
            origin: Vector3::zero(),
            direction: self.index_to_dir(index),
        };

        self.get_color(&ray)
    }
}
