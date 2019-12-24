mod cube;
mod plane;
mod sphere;

use crate::core::{BoundingVolume, Material, Transformed};
use crate::object3d::Object3D;
use crate::ray_intersection::{Intersection, Ray};
use nalgebra::{Point3, Unit, Vector3};
use std::fmt::Debug;
use std::marker::{Send, Sync};

pub use cube::*;
pub use plane::*;
pub use sphere::*;

pub trait Intersectable {
    fn make_bounding_volume(&self) -> Option<BoundingVolume>;

    fn get_material(&self) -> Material;
    fn get_children(&self) -> Option<&Vec<Object3D>>;

    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>>;
}

#[typetag::deserialize(tag = "type")]
pub trait Primitive: Send + Sync + Debug + Transformed + Intersectable {}

#[typetag::deserialize(name = "cube")]
impl Primitive for Cube {}
#[typetag::deserialize(name = "plane")]
impl Primitive for Plane {}
#[typetag::deserialize(name = "sphere")]
impl Primitive for Sphere {}
