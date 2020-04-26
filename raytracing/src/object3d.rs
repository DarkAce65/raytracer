use crate::core::{BoundingVolume, Bounds, Texture};
use crate::primitives::Primitive;
use crate::ray_intersection::{Intersection, Ray};
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering::Equal;
use std::collections::HashMap;
use std::path::Path;

fn compute_bounding_box(object: &dyn Primitive) -> Option<BoundingVolume> {
    let bounding_box = object.make_bounding_volume();

    match bounding_box {
        Bounds::Unbounded => None,
        Bounds::Children => {
            if let Some(children) = object.get_children() {
                if !children.is_empty() {
                    return children[1..]
                        .iter()
                        .fold(children[0].bounding_box, |acc, child| {
                            BoundingVolume::merge(acc, child.bounding_box)
                        });
                }
            }

            None
        }
        Bounds::Bounded(bounding_box) => {
            if let Some(children) = object.get_children() {
                children.iter().fold(Some(bounding_box), |acc, child| {
                    BoundingVolume::merge(acc, child.bounding_box)
                })
            } else {
                Some(bounding_box)
            }
        }
    }
}

#[derive(Debug)]
pub struct Object3D {
    object: Box<dyn Primitive>,
    bounding_box: Option<BoundingVolume>,
}

impl Object3D {
    pub fn new(object: Box<dyn Primitive>) -> Self {
        let bounding_box = compute_bounding_box(object.as_ref());
        Self {
            object,
            bounding_box,
        }
    }

    pub fn load_assets(
        &mut self,
        asset_base: &Path,
        textures: &mut HashMap<String, Texture>,
    ) -> bool {
        let mut should_recompute_bb = self.object.load_assets(asset_base, textures);

        if let Some(children) = self.object.get_children_mut() {
            for child in children.iter_mut() {
                if child.load_assets(asset_base, textures) {
                    should_recompute_bb = true;
                }
            }
        }

        if should_recompute_bb {
            self.bounding_box = compute_bounding_box(self.object.as_ref());
        }

        should_recompute_bb
    }

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
            .flat_map(|children| children.iter().filter_map(|object| object.intersect(ray)));

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
        Ok(Object3D::new(object))
    }
}
