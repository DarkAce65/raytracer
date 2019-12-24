use crate::core::BoundingVolume;
use crate::primitives::Primitive;
use crate::ray_intersection::{Intersection, Ray};
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering::Equal;

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

        let child_intersections = self
            .object
            .get_children()
            .into_iter()
            .flat_map(|children| children.iter().filter_map(|object| object.intersect(&ray)));

        self.object
            .intersect(ray)
            .into_iter()
            .chain(child_intersections)
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal))
    }
}

impl<'de> Deserialize<'de> for Object3D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let object: Box<dyn Primitive> = Deserialize::deserialize(deserializer)?;

        let bounding_box = object.make_bounding_volume();
        let bounding_box = if let Some(children) = object.get_children() {
            children.iter().fold(bounding_box, |acc, child| {
                BoundingVolume::merge(acc, child.bounding_box)
            })
        } else {
            bounding_box
        };

        Ok(Self {
            object,
            bounding_box,
        })
    }
}
