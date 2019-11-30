use derive_builder::Builder;
use nalgebra::Vector3;
use num_traits::identities::Zero;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug)]
pub enum MaterialSide {
    Front,
    Back,
}

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct Material {
    pub side: MaterialSide,
    pub color: Vector3<f64>,
    pub emissive: Vector3<f64>,
    pub specular: Vector3<f64>,
    pub shininess: f64,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            side: MaterialSide::Front,
            color: Vector3::zero(),
            emissive: Vector3::zero(),
            specular: Vector3::zero(),
            shininess: 30.0,
        }
    }
}
