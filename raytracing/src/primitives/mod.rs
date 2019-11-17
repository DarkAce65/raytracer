mod sphere;

use crate::raytrace::Ray;
use nalgebra::{Vector3, Vector4};
use std::marker::{Send, Sync};

pub use sphere::*;

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
    pub distance: f32,
    pub object: Box<dyn Primitive>,
}

pub trait Drawable {
    fn color(&self) -> Vector4<f32>;
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn surface_normal(&self, hit_point: &Vector3<f32>) -> Vector3<f32>;
}

pub trait Primitive: Send + Sync + Drawable + Intersectable {}
impl<T> Primitive for T where T: Send + Sync + Drawable + Intersectable {}
