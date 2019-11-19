use super::{quadratic, Drawable, Intersectable, Intersection};
use crate::raytrace::{Object3D, Ray};
use derive_builder::Builder;
use nalgebra::{Vector3, Vector4};
use num_traits::identities::Zero;

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct Sphere {
    radius: f32,
    center: Vector3<f32>,
    color: Vector4<f32>,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            radius: 1.0,
            center: Vector3::zero(),
            color: Vector4::from([1.0; 4]),
        }
    }
}

impl Object3D for Sphere {
    fn position(&self) -> Vector3<f32> {
        self.center
    }

    fn scale(&self) -> Vector3<f32> {
        unimplemented!()
    }

    fn rotation(&self) -> Vector3<f32> {
        unimplemented!()
    }
}

impl Drawable for Sphere {
    fn color(&self) -> Vector4<f32> {
        self.color
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let hypot = ray.origin - self.center;
        let ray_proj = hypot.dot(&ray.direction);
        let a = ray.direction.magnitude_squared();
        let b = 2.0 * ray_proj;
        let c = hypot.magnitude_squared() - self.radius * self.radius;

        if let Some((t0, t1)) = quadratic(a, b, c) {
            let t = if t0 < 0.0 { t1 } else { t0 };

            if t < 0.0 {
                return None;
            }

            let distance = t;
            return Some(Intersection {
                distance,
                object: Box::new(*self),
            });
        }

        None
    }

    fn surface_normal(&self, hit_point: &Vector3<f32>) -> Vector3<f32> {
        (hit_point - self.center).normalize()
    }
}
