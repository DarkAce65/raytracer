use super::LightColor;
use crate::core::{Transform, Transformed};
use nalgebra::Vector3;
use num_traits::identities::Zero;
use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
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

impl Transformed for AmbientLight {
    fn get_transform(&self) -> Transform {
        unimplemented!()
    }
}

impl LightColor for AmbientLight {
    fn get_color(&self) -> Vector3<f64> {
        self.color
    }
}
