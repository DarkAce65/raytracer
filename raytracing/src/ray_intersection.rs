use crate::primitives::Primitive;
use nalgebra::{Affine3, Point3, Vector3};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RayType {
    Primary,
    Secondary(u8),
    Shadow,
}

#[derive(Debug)]
pub struct Ray {
    pub ray_type: RayType,
    pub origin: Point3<f64>,
    pub direction: Vector3<f64>,
    pub refractive_index: f64,
}

impl Ray {
    pub fn get_depth(&self) -> u8 {
        match self.ray_type {
            RayType::Primary => 0,
            RayType::Secondary(depth) => depth,
            RayType::Shadow => panic!("shadow rays have no depth"),
        }
    }

    pub fn transform(&self, transform: Affine3<f64>) -> Ray {
        let origin = transform * self.origin;
        let direction = transform * self.direction;

        Ray {
            ray_type: self.ray_type,
            origin,
            direction,
            refractive_index: self.refractive_index,
        }
    }
}

#[derive(Debug)]
pub struct Intersection<'a> {
    pub distance: f64,
    pub object: &'a dyn Primitive,
}
