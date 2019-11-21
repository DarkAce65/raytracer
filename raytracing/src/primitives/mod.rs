mod cube;
mod plane;
mod sphere;

use crate::core::{Intersection, Object3D, Ray};
use nalgebra::{Point3, Unit, Vector3, Vector4};
use std::fmt::Debug;
use std::marker::{Send, Sync};

pub use cube::*;
pub use plane::*;
pub use sphere::*;

pub trait Drawable {
    fn color(&self) -> Vector4<f64>;
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>>;
}

pub trait Primitive: Send + Sync + Debug + Object3D + Drawable + Intersectable {}
impl<T> Primitive for T where T: Send + Sync + Debug + Object3D + Drawable + Intersectable {}
