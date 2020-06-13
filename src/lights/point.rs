use crate::core::{Transform, Transformed};
use nalgebra::Vector3;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PointLight {
    transform: Transform,
    pub color: Vector3<f64>,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            color: Vector3::from([1.0; 3]),
        }
    }
}

impl PointLight {
    pub fn new(color: Vector3<f64>, transform: Transform) -> Self {
        Self { color, transform }
    }
}

impl Transformed for PointLight {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
}
