mod cube;
mod group;
mod mesh;
mod plane;
mod sphere;
mod triangle;

use crate::core::Texture;
use crate::core::{BoundedObject, Material, Transform, Transformed};
use crate::ray_intersection::{IntermediateData, Intersectable};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::{Send, Sync};
use std::path::Path;

pub use cube::*;
pub use group::*;
pub use mesh::*;
pub use plane::*;
pub use sphere::*;
pub use triangle::*;

pub trait HasMaterial {
    fn get_material(&self) -> &Material;
    fn get_material_mut(&mut self) -> &mut Material;

    fn load_textures(&mut self, asset_base: &Path, textures: &mut HashMap<String, Texture>) {
        self.get_material_mut().load_textures(asset_base, textures);
    }
}

pub trait Loadable: HasMaterial {
    fn load_assets(&mut self, asset_base: &Path, textures: &mut HashMap<String, Texture>) -> bool {
        self.load_textures(asset_base, textures);

        false
    }
}

pub trait Primitive {
    fn into_bounded_object(self: Box<Self>, parent_transform: &Transform) -> Option<BoundedObject>;

    fn get_children(&self) -> Option<&Vec<Box<dyn Object3D>>>;
    fn get_children_mut(&mut self) -> Option<&mut Vec<Box<dyn Object3D>>>;

    fn surface_normal(
        &self,
        object_hit_point: &Point3<f64>,
        intermediate: IntermediateData,
    ) -> Unit<Vector3<f64>>;
    fn uv(
        &self,
        object_hit_point: &Point3<f64>,
        object_normal: &Unit<Vector3<f64>>,
        intermediate: IntermediateData,
    ) -> Vector2<f64>;
}

#[typetag::deserialize(tag = "type")]
pub trait Object3D:
    Send + Sync + Debug + Transformed + Intersectable + Primitive + HasMaterial + Loadable
{
}

#[typetag::deserialize(name = "cube")]
impl Object3D for Cube {}
#[typetag::deserialize(name = "plane")]
impl Object3D for Plane {}
#[typetag::deserialize(name = "sphere")]
impl Object3D for Sphere {}
#[typetag::deserialize(name = "triangle")]
impl Object3D for Triangle {}

#[typetag::deserialize(name = "group")]
impl Object3D for Group {}

#[typetag::deserialize(name = "mesh")]
impl Object3D for Mesh {}
