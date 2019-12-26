mod cube;
mod plane;
mod sphere;
mod triangle;

use crate::core::{BoundingVolume, Material, Transformed};
use crate::object3d::Object3D;
use crate::ray_intersection::{Intersection, Ray};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use std::fmt::Debug;
use std::marker::{Send, Sync};

pub use cube::*;
pub use plane::*;
pub use sphere::*;
pub use triangle::*;

pub trait Intersectable {
    fn make_bounding_volume(&self) -> Option<BoundingVolume>;

    fn get_material(&self) -> &Material;
    fn get_material_mut(&mut self) -> &mut Material;
    fn get_children(&self) -> Option<&Vec<Object3D>>;
    fn get_children_mut(&mut self) -> Option<&mut Vec<Object3D>>;

    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>>;
    fn uv(&self, hit_point: &Point3<f64>, normal: &Unit<Vector3<f64>>) -> Vector2<f64>;
}

#[typetag::deserialize(tag = "type")]
pub trait Primitive: Send + Sync + Debug + Transformed + Intersectable {}

#[typetag::deserialize(name = "cube")]
impl Primitive for Cube {}
#[typetag::deserialize(name = "plane")]
impl Primitive for Plane {}
#[typetag::deserialize(name = "sphere")]
impl Primitive for Sphere {}
#[typetag::deserialize(name = "triangle")]
impl Primitive for Triangle {}
