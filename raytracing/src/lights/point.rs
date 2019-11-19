use crate::raytrace::Object3D;
use derive_builder::Builder;
use nalgebra::{Point3, Vector3};

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct PointLight {
    position: Point3<f64>,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
        }
    }
}

impl Object3D for PointLight {
    fn position(&self) -> Point3<f64> {
        self.position
    }

    fn scale(&self) -> Vector3<f64> {
        unimplemented!()
    }

    fn rotation(&self) -> Vector3<f64> {
        unimplemented!()
    }
}
