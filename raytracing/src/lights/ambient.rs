use super::LightColor;
use crate::core::Object3D;
use derive_builder::Builder;
use nalgebra::{Point3, Vector3};
use num_traits::identities::Zero;

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct AmbientLight {
    color: Vector3<f64>,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: Vector3::zero(),
        }
    }
}

impl Object3D for AmbientLight {
    fn position(&self) -> Point3<f64> {
        unimplemented!()
    }
}

impl LightColor for AmbientLight {
    fn get_color(&self) -> Vector3<f64> {
        self.color
    }
}
