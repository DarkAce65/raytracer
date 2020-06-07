use super::{Object3D, RaytracingObject};
use crate::core::Transform;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    #[serde(default)]
    transform: Transform,

    pub children: Vec<Object3D>,
}

impl Group {
    pub fn flatten_to_world(self, transform: &Transform) -> Vec<Box<dyn RaytracingObject>> {
        let transform = transform * self.transform;
        let mut objects: Vec<Box<dyn RaytracingObject>> = Vec::new();

        for child in self.children {
            let child_objects: Vec<Box<dyn RaytracingObject>> = child.flatten_to_world(&transform);
            objects.extend(child_objects);
        }

        objects
    }
}
