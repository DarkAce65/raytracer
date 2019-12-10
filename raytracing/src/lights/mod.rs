mod ambient;
mod point;

use crate::core::Transformed;
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

#[typetag::deserialize(tag = "type")]
pub trait Light: Send + Sync + Debug + Transformed + LightColor {
    fn get_type(&self) -> LightType;
}

#[typetag::deserialize(name = "ambient")]
impl Light for AmbientLight {
    fn get_type(&self) -> LightType {
        LightType::Ambient
    }
}

#[typetag::deserialize(name = "point")]
impl Light for PointLight {
    fn get_type(&self) -> LightType {
        LightType::Point
    }
}
