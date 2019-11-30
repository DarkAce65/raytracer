mod ambient;
mod point;

use crate::core::Object3D;
use nalgebra::Vector3;
use std::fmt::Debug;
use std::marker::{Send, Sync};

pub use ambient::*;
pub use point::*;

pub enum LightType {
    Ambient,
    Point,
}

pub trait LightColor {
    fn get_color(&self) -> Vector3<f64>;
}

#[typetag::serde(tag = "type")]
pub trait Light: Send + Sync + Debug + Object3D + LightColor {
    fn get_type(&self) -> LightType;
}

#[typetag::serde(name = "ambient")]
impl Light for AmbientLight {
    fn get_type(&self) -> LightType {
        LightType::Ambient
    }
}

#[typetag::serde(name = "point")]
impl Light for PointLight {
    fn get_type(&self) -> LightType {
        LightType::Point
    }
}
