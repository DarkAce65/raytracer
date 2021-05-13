use crate::core::{Transform, Transformed};
use nalgebra::Vector3;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PointLight {
    transform: Transform,
    color: Vector3<f64>,
    intensity: f64,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            color: Vector3::from([1.0; 3]),
            intensity: 10.0,
        }
    }
}

impl PointLight {
    pub fn new(color: Vector3<f64>, intensity: f64, transform: Transform) -> Self {
        Self {
            transform,
            color,
            intensity,
        }
    }

    pub fn get_color(&self, distance: f64) -> Vector3<f64> {
        (self.intensity * self.color / distance.powi(2)).map(|c| c.clamp(0.0, 1.0))
    }
}

impl Transformed for PointLight {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
}
