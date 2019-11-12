use crate::raytrace::Ray;
use image::Rgba;
use nalgebra::Vector3;
use num_traits::identities::Zero;

pub trait Drawable {
    fn color(&self) -> Rgba<u8>;
}

pub trait Intersectable {
    fn intersects(&self, ray: &Ray) -> bool;
}

pub trait Primitive: Drawable + Intersectable {}
impl<T> Primitive for T where T: Drawable + Intersectable {}

#[derive(Debug)]
pub struct Sphere {
    radius: f32,
    center: Vector3<f32>,
}

#[allow(dead_code)]
impl Sphere {
    pub fn new() -> Self {
        Sphere {
            radius: 10.0,
            center: Vector3::zero(),
        }
    }

    pub fn from(radius: f32, center: Vector3<f32>) -> Self {
        Sphere { radius, center }
    }
}

impl Drawable for Sphere {
    fn color(&self) -> Rgba<u8> {
        Rgba([255, 0, 0, 255])
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
