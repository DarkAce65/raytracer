use crate::raytrace::Ray;
use derive_builder::Builder;
use nalgebra::Vector3;
use num_traits::identities::Zero;
use std::marker::{Send, Sync};

pub trait Drawable {
    fn color(&self) -> [u8; 4];
}

pub trait Intersectable {
    fn intersects(&self, ray: &Ray) -> bool;
}

pub trait Primitive: Send + Sync + Drawable + Intersectable {}
impl<T> Primitive for T where T: Send + Sync + Drawable + Intersectable {}

#[derive(Builder, Debug)]
#[builder(default)]
pub struct Sphere {
    radius: f32,
    center: Vector3<f32>,
    color: [u8; 4],
}

impl Default for Sphere {
    fn default() -> Self {
        Sphere {
            radius: 1.0,
            center: Vector3::zero(),
            color: [255; 4],
        }
    }
}

impl Drawable for Sphere {
    fn color(&self) -> [u8; 4] {
        self.color
    }
}

impl Intersectable for Sphere {
    fn intersects(&self, ray: &Ray) -> bool {
        let l: Vector3<f32> = self.center - ray.origin;
        let adj2 = l.dot(&ray.direction);
        let d2 = l.dot(&l) - (adj2 * adj2);
        d2 < (self.radius * self.radius)
    }
}
