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
use serde::Deserialize;
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, tag = "type", rename_all = "lowercase")]
pub enum SemanticObject {
    Cube(SemanticCube),
    Plane(SemanticPlane),
    Sphere(SemanticSphere),
    Triangle(SemanticTriangle),
    Mesh(SemanticMesh),
    Group(SemanticGroup),
}

impl SemanticObject {
    pub fn load_assets(
        object: &mut SemanticObject,
        asset_base: &Path,
        textures: &mut HashMap<String, Texture>,
    ) {
        match object {
            SemanticObject::Cube(semantic) => {
                semantic.material.load_textures(asset_base, textures);
            }
            SemanticObject::Plane(semantic) => {
                semantic.material.load_textures(asset_base, textures);
            }
            SemanticObject::Sphere(semantic) => {
                semantic.material.load_textures(asset_base, textures);
            }
            SemanticObject::Triangle(semantic) => {
                semantic.material.load_textures(asset_base, textures);
            }
            SemanticObject::Mesh(semantic) => {
                semantic.load_assets(asset_base);
                semantic.material.load_textures(asset_base, textures);
            }
            SemanticObject::Group(_) => {}
        }

        if let Some(children) = object.get_children_mut() {
            for child in children {
                SemanticObject::load_assets(child, asset_base, textures);
            }
        }
    }

    pub fn get_children_mut(&mut self) -> Option<&mut Vec<SemanticObject>> {
        match self {
            SemanticObject::Cube(semantic) => semantic.children.as_mut(),
            SemanticObject::Triangle(semantic) => semantic.children.as_mut(),
            SemanticObject::Plane(semantic) => semantic.children.as_mut(),
            SemanticObject::Sphere(semantic) => semantic.children.as_mut(),
            SemanticObject::Mesh(semantic) => semantic.children.as_mut(),
            SemanticObject::Group(semantic) => Some(&mut semantic.children),
        }
    }

    pub fn flatten_to_world(self, transform: &Transform) -> Vec<Box<dyn Object3D>> {
        match self {
            SemanticObject::Cube(semantic) => semantic.flatten_to_world(transform),
            SemanticObject::Triangle(semantic) => semantic.flatten_to_world(transform),
            SemanticObject::Plane(semantic) => semantic.flatten_to_world(transform),
            SemanticObject::Sphere(semantic) => semantic.flatten_to_world(transform),
            SemanticObject::Mesh(semantic) => semantic.flatten_to_world(transform),
            SemanticObject::Group(semantic) => semantic.flatten_to_world(transform),
        }
    }
}

pub trait HasMaterial {
    fn get_material(&self) -> &Material;
    fn get_material_mut(&mut self) -> &mut Material;
}

pub trait Loadable: HasMaterial {
    fn load_assets(&mut self, asset_base: &Path, textures: &mut HashMap<String, Texture>) -> bool {
        // self.load_textures(asset_base, textures);

        false
    }
}

pub trait Primitive: Transformed {
    fn into_bounded_object(self: Box<Self>) -> Option<BoundedObject>;
    fn into_bounded_object_tree(
        self: Box<Self>,
        parent_transform: &Transform,
    ) -> Vec<BoundedObject> {
        let transform = parent_transform * self.get_transform();

        let mut bounded_objects = Vec::new();

        // if let Some(children) = self.get_children() {
        //     for child in children {
        //         bounded_objects.append(&mut child.into_bounded_object_tree(&transform));
        //     }
        // }

        let bounded_object = self.into_bounded_object();
        if let Some(bounded_object) = bounded_object {
            bounded_objects.push(bounded_object);
        }

        bounded_objects
    }

    // fn get_children(self: Box<Self>) -> Option<Vec<Box<dyn Object3D>>>;
    // fn get_children_ref(&self) -> Option<&Vec<Box<dyn Object3D>>>;
    // fn get_children_mut(&mut self) -> Option<&mut Vec<Box<dyn Object3D>>>;

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

pub trait Object3D:
    Send + Sync + Debug + Transformed + Intersectable + Primitive + HasMaterial + Loadable
{
}

impl Object3D for Cube {}
impl Object3D for Plane {}
impl Object3D for Sphere {}
impl Object3D for Triangle {}
