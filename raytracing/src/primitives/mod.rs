mod cube;
mod mesh;
mod plane;
mod sphere;
mod triangle;

use crate::core::{Bounds, Material, Transformed};
use crate::object3d::Object3D;
use crate::ray_intersection::{Intersection, Ray};
use std::fmt::Debug;
use std::marker::{Send, Sync};
use std::path::Path;

pub use cube::*;
pub use mesh::*;
pub use plane::*;
pub use sphere::*;
pub use triangle::*;

pub trait Loadable {
    fn load_assets(&mut self, _asset_base: &Path) -> bool {
        false
    }
}

pub trait Intersectable {
    fn make_bounding_volume(&self) -> Bounds;

    fn get_material(&self) -> &Material;
    fn get_material_mut(&mut self) -> &mut Material;
    fn get_children(&self) -> Option<&Vec<Object3D>>;
    fn get_children_mut(&mut self) -> Option<&mut Vec<Object3D>>;

    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

#[typetag::deserialize(tag = "type")]
pub trait Primitive: Send + Sync + Debug + Loadable + Transformed + Intersectable {}

#[typetag::deserialize(name = "cube")]
impl Primitive for Cube {}
#[typetag::deserialize(name = "plane")]
impl Primitive for Plane {}
#[typetag::deserialize(name = "sphere")]
impl Primitive for Sphere {}
#[typetag::deserialize(name = "triangle")]
impl Primitive for Triangle {}

#[typetag::deserialize(name = "mesh")]
impl Primitive for Mesh {}
