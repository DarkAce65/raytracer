use crate::raytrace::Object3D;
use derive_builder::Builder;
use nalgebra::Vector3;
use num_traits::identities::Zero;

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct PointLight {
    position: Vector3<f32>,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
        }
    }
}

impl Object3D for PointLight {
    fn position(&self) -> Vector3<f32> {
        self.position
    }

    fn scale(&self) -> Vector3<f32> {
        unimplemented!()
    }

    fn rotation(&self) -> Vector3<f32> {
        unimplemented!()
    }
}
