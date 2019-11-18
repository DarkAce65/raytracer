use super::{Drawable, Intersectable, Intersection};
use crate::raytrace::Ray;
use derive_builder::Builder;
use nalgebra::{Vector3, Vector4};
use num_traits::identities::Zero;

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct Cube {
    size: f32,
    center: Vector3<f32>,
    color: Vector4<f32>,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            size: 1.0,
            center: Vector3::zero(),
            color: Vector4::from([1.0; 4]),
        }
    }
}

impl Drawable for Cube {
    fn color(&self) -> Vector4<f32> {
        self.color
    }
}

impl Intersectable for Cube {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let half = self.size / 2.0;

        let mut t0 =
            (self.center.x - ray.direction.x.signum() * half - ray.origin.x) / ray.direction.x;
        let mut t1 =
            (self.center.x + ray.direction.x.signum() * half - ray.origin.x) / ray.direction.x;
        let tymin =
            (self.center.y - ray.direction.y.signum() * half - ray.origin.y) / ray.direction.y;
        let tymax =
            (self.center.y + ray.direction.y.signum() * half - ray.origin.y) / ray.direction.y;

        if t0 > tymax || t1 < tymin {
            return None;
        }

        if tymin > t0 {
            t0 = tymin;
        }
        if tymax < t1 {
            t1 = tymax;
        }

        let tzmin =
            (self.center.z - ray.direction.z.signum() * half - ray.origin.z) / ray.direction.z;
        let tzmax =
            (self.center.z + ray.direction.z.signum() * half - ray.origin.z) / ray.direction.z;

        if t0 > tzmax || t1 < tzmin {
            return None;
        }

        if tzmin > t0 {
            t0 = tzmin;
        }
        if tzmax < t1 {
            t1 = tzmax;
        }

        let t = if t0 < 0.0 { t1 } else { t0 };
        if t < 0.0 {
            return None;
        }

        let intersection = Intersection {
            distance: t,
            object: Box::new(*self),
        };

        Some(intersection)
    }

    fn surface_normal(&self, hit_point: &Vector3<f32>) -> Vector3<f32> {
        let normal = hit_point - self.center;
        if normal.x > normal.y {
            if normal.x > normal.z {
                Vector3::x()
            } else {
                Vector3::z()
            }
        } else if normal.y > normal.z {
            Vector3::y()
        } else {
            Vector3::z()
        }
    }
}
