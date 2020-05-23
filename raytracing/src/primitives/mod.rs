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
pub enum Object3D {
    Cube(Box<Cube>),
    Plane(Box<Plane>),
    Sphere(Box<Sphere>),
    Triangle(Box<Triangle>),
    Mesh(Box<Mesh>),
    Group(Box<Group>),
}

impl Object3D {
    pub fn load_assets(
        object: &mut Object3D,
        asset_base: &Path,
        textures: &mut HashMap<String, Texture>,
    ) {
        match object {
            Object3D::Cube(semantic) => semantic.material.load_textures(asset_base, textures),
            Object3D::Plane(semantic) => semantic.material.load_textures(asset_base, textures),
            Object3D::Sphere(semantic) => semantic.material.load_textures(asset_base, textures),
            Object3D::Triangle(semantic) => semantic.material.load_textures(asset_base, textures),
            Object3D::Mesh(semantic) => {
                semantic.load_assets(asset_base);
                semantic.material.load_textures(asset_base, textures);
            }
            Object3D::Group(_) => {}
        }

        if let Some(children) = object.get_children_mut() {
            for child in children {
                Object3D::load_assets(child, asset_base, textures);
            }
        }
    }

    fn get_children_mut(&mut self) -> Option<&mut Vec<Object3D>> {
        match self {
            Object3D::Cube(semantic) => semantic.children.as_mut(),
            Object3D::Triangle(semantic) => semantic.children.as_mut(),
            Object3D::Plane(semantic) => semantic.children.as_mut(),
            Object3D::Sphere(semantic) => semantic.children.as_mut(),
            Object3D::Mesh(semantic) => semantic.children.as_mut(),
            Object3D::Group(semantic) => Some(&mut semantic.children),
        }
    }

    pub fn flatten_to_world(self, transform: &Transform) -> Vec<Box<dyn RaytracingObject>> {
        match self {
            Object3D::Cube(semantic) => semantic.flatten_to_world(transform),
            Object3D::Triangle(semantic) => semantic.flatten_to_world(transform),
            Object3D::Plane(semantic) => semantic.flatten_to_world(transform),
            Object3D::Sphere(semantic) => semantic.flatten_to_world(transform),
            Object3D::Mesh(semantic) => semantic.flatten_to_world(transform),
            Object3D::Group(semantic) => semantic.flatten_to_world(transform),
        }
    }
}

pub trait HasMaterial {
    fn get_material(&self) -> &Material;
}

pub trait Primitive: Transformed {
    fn into_bounded_object(self: Box<Self>) -> Option<BoundedObject>;

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

pub trait RaytracingObject:
    Send + Sync + Debug + Transformed + Intersectable + Primitive + HasMaterial
{
}

impl RaytracingObject for RaytracingCube {}
impl RaytracingObject for RaytracingPlane {}
impl RaytracingObject for RaytracingSphere {}
impl RaytracingObject for RaytracingTriangle {}
