use crate::lights::Light;
use crate::primitives::Primitive;
use crate::scene::{Camera, Scene};
use nalgebra::{Point3, Vector3};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneJSON {
    lights: Vec<Box<dyn Light>>,
    objects: Vec<Box<dyn Primitive>>,
}

impl SceneJSON {
    pub fn into_scene(self) -> Scene {
        Scene {
            width: 800,
            height: 800,
            camera: Camera::from(
                65.0,
                Point3::from([2.0, 5.0, 15.0]),
                Point3::origin(),
                Vector3::y_axis(),
            ),
            lights: self.lights,
            objects: self.objects,
        }
    }
}
