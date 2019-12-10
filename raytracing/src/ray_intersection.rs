use crate::primitives::Primitive;
use nalgebra::{Affine3, Point3, Vector3};

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f64>,
    pub direction: Vector3<f64>,
    pub refractive_index: f64,
}

impl Ray {
    pub fn transform(&self, transform: Affine3<f64>) -> Ray {
        let origin = transform * self.origin;
        let direction = transform * self.direction;

        Ray {
            origin,
            direction,
            refractive_index: self.refractive_index,
        }
    }
}

#[derive(Debug)]
pub struct Intersection {
    pub distance: f64,
    pub object: Box<dyn Primitive>,
}
