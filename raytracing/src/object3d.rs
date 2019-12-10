use crate::core::BoundingVolume;
use crate::primitives::Primitive;
use crate::ray_intersection::{Intersection, Ray};
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct Object3D {
    object: Box<dyn Primitive>,
    bounding_box: Option<BoundingVolume>,
}

impl Object3D {
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        if let Some(bounding_box) = &self.bounding_box {
            if !bounding_box.intersect(ray) {
                return None;
            }
        }

        self.object.intersect(ray)
    }
}

impl<'de> Deserialize<'de> for Object3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let object: Box<dyn Primitive> = Deserialize::deserialize(deserializer)?;
        let bounding_box = object.make_bounding_volume();

        Ok(Self {
            object,
            bounding_box,
        })
    }
}
