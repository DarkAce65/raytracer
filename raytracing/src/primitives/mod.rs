mod cube;
mod plane;
mod sphere;

use crate::core::{BoundingVolume, Material, Transformed};
use crate::ray_intersection::{Intersection, Ray};
use nalgebra::{Point3, Unit, Vector3};
use std::fmt::Debug;
use std::marker::{Send, Sync};

pub use cube::*;
pub use plane::*;
pub use sphere::*;

pub trait Intersectable {
    fn make_bounding_volume(&self) -> BoundingVolume;
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>>;
}

pub trait Drawable {
    fn get_material(&self) -> Material;
}

#[typetag::deserialize(tag = "type")]
pub trait Primitive: Send + Sync + Debug + Transformed + Intersectable + Drawable {}

#[typetag::deserialize(name = "cube")]
impl Primitive for Cube {}
#[typetag::deserialize(name = "plane")]
impl Primitive for Plane {}
#[typetag::deserialize(name = "sphere")]
impl Primitive for Sphere {}
