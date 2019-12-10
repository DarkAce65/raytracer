use super::LightColor;
use crate::core::{Transform, Transformed};
use nalgebra::Vector3;
use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PointLight {
    transform: Transform,
    color: Vector3<f64>,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            color: Vector3::from([1.0; 3]),
        }
    }
}

impl Transformed for PointLight {
    fn get_transform(&self) -> Transform {
        self.transform
    }
}

impl LightColor for PointLight {
    fn get_color(&self) -> Vector3<f64> {
        self.color
    }
}
