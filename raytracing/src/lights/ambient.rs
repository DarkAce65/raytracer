use super::LightColor;
use crate::core::{Object3D, Transform};
use derive_builder::Builder;
use nalgebra::Vector3;
use num_traits::identities::Zero;
use serde::{Deserialize, Serialize};

#[derive(Builder, Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
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
    fn transform(&self) -> Transform {
        unimplemented!()
    }
}

impl LightColor for AmbientLight {
    fn get_color(&self) -> Vector3<f64> {
        self.color
    }
}
