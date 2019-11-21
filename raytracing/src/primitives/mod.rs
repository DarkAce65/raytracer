mod cube;
mod plane;
mod sphere;

use crate::core::{Intersection, Object3D, Ray};
use derive_builder::Builder;
use nalgebra::{Point3, Unit, Vector3};
use num_traits::identities::Zero;
use std::fmt::Debug;
use std::marker::{Send, Sync};

pub use cube::*;
pub use plane::*;
pub use sphere::*;

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct Material {
    pub color: Vector3<f64>,
    pub emissive: Vector3<f64>,
    pub specular: Vector3<f64>,
    pub shininess: f64,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Vector3::zero(),
            emissive: Vector3::zero(),
            specular: Vector3::zero(),
            shininess: 30.0,
        }
    }
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>>;
}

pub trait Drawable {
    fn material(&self) -> Material;
}

pub trait Primitive: Send + Sync + Debug + Object3D + Intersectable + Drawable {}

impl Primitive for Cube {}
impl Primitive for Plane {}
impl Primitive for Sphere {}
