use crate::core::{MaterialSide, Transform};
use crate::primitives::Object3D;
use nalgebra::{Affine3, Point3, Unit, Vector2, Vector3};

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

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

#[derive(Debug, Copy, Clone)]
pub enum IntermediateData {
    Empty,
    Barycentric(f64, f64, f64),
}

#[derive(Debug)]
struct IntersectionData {
    hit_point: Point3<f64>,
    normal: Unit<Vector3<f64>>,
    uv: Vector2<f64>,
}

#[derive(Debug)]
pub struct Intersection<'a> {
    pub object: &'a dyn Object3D,
    pub distance: f64,
    pub root_transform: Option<&'a Transform>,
    intermediate: IntermediateData,
    data: Option<IntersectionData>,
}

impl<'a> Intersection<'a> {
    pub fn new_with_data(
        object: &'a dyn Object3D,
        distance: f64,
        intermediate: IntermediateData,
    ) -> Self {
        Self {
            object,
            distance,
            root_transform: None,
            intermediate,
            data: None,
        }
    }

    pub fn new(object: &'a dyn Object3D, distance: f64) -> Self {
        Self::new_with_data(object, distance, IntermediateData::Empty)
    }

    pub fn compute_data(&mut self, ray: &Ray) {
        let transform = self
            .root_transform
            .expect("intersection has no root transform");

        let hit_point = ray.origin + ray.direction * self.distance;
        let object_hit_point = transform.inverse() * hit_point;

        let object_normal = self
            .object
            .surface_normal(&object_hit_point, self.intermediate);
        let normal =
            Unit::new_normalize(transform.inverse_transpose() * object_normal.into_inner());
        let normal = match self.object.get_material().side() {
            MaterialSide::Both => {
                if normal.dot(&ray.direction) > 0.0 {
                    -normal
                } else {
                    normal
                }
            }
            MaterialSide::Front => normal,
            MaterialSide::Back => -normal,
        };

        let uv = self
            .object
            .uv(&object_hit_point, &object_normal, self.intermediate);

        self.data = Some(IntersectionData {
            hit_point,
            normal,
            uv,
        });
    }

    fn get_data(&self) -> &IntersectionData {
        self.data.as_ref().expect("intersection data not computed")
    }

    pub fn get_hit_point(&self) -> Point3<f64> {
        self.get_data().hit_point
    }

    pub fn get_normal(&self) -> Unit<Vector3<f64>> {
        self.get_data().normal
    }

    pub fn get_uv(&self) -> Vector2<f64> {
        self.get_data().uv
    }
}
