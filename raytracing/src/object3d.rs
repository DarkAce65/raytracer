use crate::core::{BoundingVolume, Bounds, Texture, Transform};
use crate::primitives::Primitive;
use crate::ray_intersection::{Intersection, Ray};
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering::Equal;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct Object3D {
    object: Box<dyn Primitive>,
    bounding_box: Option<BoundingVolume>,
}

impl Object3D {
    pub fn new(object: Box<dyn Primitive>) -> Self {
        Self {
            object,
            bounding_box: None,
        }
    }

    pub fn load_assets(&mut self, asset_base: &Path, textures: &mut HashMap<String, Texture>) {
        self.object.load_assets(asset_base, textures);

        if let Some(children) = self.object.get_children_mut() {
            for child in children.iter_mut() {
                child.load_assets(asset_base, textures);
            }
        }
    }

    pub fn compute_bounding_box_with_transform(&mut self, transform: &Transform) {
        let object_transform = transform * self.object.get_transform();
        if let Some(children) = self.object.get_children_mut() {
            for child in children.iter_mut() {
                child.compute_bounding_box_with_transform(&object_transform);
            }
        }

        let object_bounds = self.object.make_bounding_volume(&object_transform);

        let mut bounding_boxes: Vec<Option<BoundingVolume>> = Vec::new();

        match object_bounds {
            Bounds::Unbounded => {}
            Bounds::Children => {
                if let Some(children) = self.object.get_children() {
                    bounding_boxes.extend(children.iter().map(|child| child.bounding_box));
                }
            }
            Bounds::Bounded(bounding_box) => {
                if let Some(children) = self.object.get_children() {
                    bounding_boxes.extend(children.iter().map(|child| child.bounding_box));
                }

                bounding_boxes.push(Some(bounding_box))
            }
        }

        self.bounding_box = if bounding_boxes.is_empty() {
            None
        } else {
            bounding_boxes[1..]
                .iter()
                .fold(bounding_boxes[0], |acc, bounding_box| {
                    BoundingVolume::merge(acc, *bounding_box)
                })
        };
    }

    pub fn compute_bounding_box(&mut self) {
        self.compute_bounding_box_with_transform(&Transform::default())
    }

    fn intersect_with_transform(&self, ray: &Ray, transform: &Transform) -> Option<Intersection> {
        if let Some(bounding_box) = &self.bounding_box {
            if !bounding_box.intersect(ray) {
                return None;
            }
        }

        let object_transform = transform * self.object.get_transform();

        let child_intersections = self.object.get_children().into_iter().flat_map(|children| {
            children
                .iter()
                .filter_map(|object| object.intersect_with_transform(ray, &object_transform))
        });

        let ray = &ray.transform(object_transform.inverse());

        self.object
            .intersect(ray)
            .map(|mut intersection| {
                intersection.root_transform = object_transform.clone();
                intersection
            })
            .into_iter()
            .chain(child_intersections)
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal))
    }

    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        self.intersect_with_transform(ray, &Transform::default())
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
