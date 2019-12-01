mod cube;
mod plane;
mod sphere;

use crate::core::{Intersection, Material, Object3D, Ray};
use nalgebra::{Point3, Unit, Vector3};
use std::fmt::Debug;
use std::marker::{Send, Sync};

pub use cube::*;
pub use plane::*;
pub use sphere::*;

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>>;
}

pub trait Drawable {
    fn material(&self) -> Material;
}

#[typetag::serde(tag = "type")]
pub trait Primitive: Send + Sync + Debug + Object3D + Intersectable + Drawable {}

#[typetag::serde(name = "cube")]
impl Primitive for Cube {}
#[typetag::serde(name = "plane")]
impl Primitive for Plane {}
#[typetag::serde(name = "sphere")]
impl Primitive for Sphere {}
