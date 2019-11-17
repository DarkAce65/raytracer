use crate::raytrace::Ray;
use derive_builder::Builder;
use nalgebra::{Vector3, Vector4};
use num_traits::identities::Zero;
use std::marker::{Send, Sync};

fn quadratic(a: f32, b: f32, c: f32) -> Option<(f32, f32)> {
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        None
    } else if discriminant == 0.0 {
        Some((-0.5 * b / a, -0.5 * b / a))
    } else {
        let q = -0.5 * (b + b.signum() * discriminant.sqrt());
        let r0 = q / a;
        let r1 = c / q;
        Some((r0.min(r1), r0.max(r1)))
    }
}

pub struct Intersection {
    pub hit_point: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub distance: f32,
    pub object: Box<dyn Primitive>,
}

pub trait Drawable {
    fn color(&self) -> Vector4<f32>;
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

pub trait Primitive: Send + Sync + Drawable + Intersectable {}
impl<T> Primitive for T where T: Send + Sync + Drawable + Intersectable {}

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

            let hit_point = ray.origin + ray.direction * t;
            let normal = (hit_point - self.center).normalize();
            let distance = hit_point.magnitude();
            return Some(Intersection {
                hit_point,
                normal,
                distance,
                object: Box::new(*self),
            });
        }

        None
    }
}
