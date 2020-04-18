use crate::core::MaterialSide;
use crate::primitives::Primitive;
use nalgebra::{Affine3, Point3, Unit, Vector2, Vector3};

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
    pub object: &'a dyn Primitive,
    pub distance: f64,
    pub hit_point: Point3<f64>,
    pub normal: Unit<Vector3<f64>>,
    pub uv: Vector2<f64>,
}

impl<'a> Intersection<'a> {
    pub fn get_hit_point(&self) -> Point3<f64> {
        self.object.get_transform().matrix() * self.hit_point
    }

    pub fn get_normal(&self, ray: &Ray) -> Unit<Vector3<f64>> {
        let normal = Unit::new_normalize(
            self.object.get_transform().inverse_transpose() * self.normal.into_inner(),
        );

        match self.object.get_material().side() {
            MaterialSide::Both => {
                if normal.dot(&ray.direction) > 0.0 {
                    -normal
                } else {
                    normal
                }
            }
            MaterialSide::Front => normal,
            MaterialSide::Back => -normal,
        }
    }
}
