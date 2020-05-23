use super::{Object3D, SemanticObject};
use crate::core::Transform;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticGroup {
    #[serde(default)]
    transform: Transform,

    pub children: Vec<SemanticObject>,
}

impl SemanticGroup {
    pub fn flatten_to_world(self, transform: &Transform) -> Vec<Box<dyn Object3D>> {
        let transform = transform * self.transform;

        let mut objects: Vec<Box<dyn Object3D>> = Vec::new();

        for child in self.children {
            let child_objects: Vec<Box<dyn Object3D>> = child.flatten_to_world(&transform);
            objects.extend(child_objects);
        }

        objects
    }
}
