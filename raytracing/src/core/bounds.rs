use super::Transform;
use crate::primitives::RaytracingObject;
use crate::ray_intersection::{Intersectable, Intersection, Ray};
use nalgebra::Point3;
use std::cmp::Ordering::Equal;
use std::fmt;

fn build_bounding_volume(objects: &[BoundedObject]) -> BoundingVolume {
    let objects: Vec<&BoundedObject> = objects
        .iter()
        .filter(|object| match object {
            BoundedObject::Bounded(_, _) => true,
            BoundedObject::Unbounded(_) => false,
        })
        .collect();

    if objects.is_empty() {
        panic!("trying to build a bounding volume out of nothing")
    }

    let bounding_volume = match objects[0] {
        BoundedObject::Bounded(_, bounding_volume) => bounding_volume,
        BoundedObject::Unbounded(_) => unreachable!(),
    };

    objects[1..]
        .iter()
        .fold(*bounding_volume, |acc, object| match object {
            BoundedObject::Bounded(_, bounding_volume) => {
                BoundingVolume::merge(&acc, bounding_volume)
            }
            _ => unreachable!(),
        })
}

#[derive(Copy, Clone, Debug)]
pub struct BoundingVolume {
    center: Point3<f64>,
    bounds_min: Point3<f64>,
    bounds_max: Point3<f64>,
}

impl BoundingVolume {
    pub fn from_bounds(bounds_min: Point3<f64>, bounds_max: Point3<f64>) -> Self {
        assert!(bounds_max >= bounds_min);

        Self {
            center: nalgebra::center(&bounds_min, &bounds_max),
            bounds_min,
            bounds_max,
        }
    }

    pub fn from_bounds_and_transform(
        bounds_min: Point3<f64>,
        bounds_max: Point3<f64>,
        transform: &Transform,
    ) -> Self {
        assert!(bounds_max >= bounds_min);

        let mut vertices = [Point3::origin(); 8];
        let mut i = 0;
        for x in &[bounds_min.x, bounds_max.x] {
            for y in &[bounds_min.y, bounds_max.y] {
                for z in &[bounds_min.z, bounds_max.z] {
                    vertices[i] = Point3::new(*x, *y, *z);
                    i += 1;
                }
            }
        }

        let mut min = transform.matrix() * vertices[0];
        let mut max = min;
        for vertex in vertices[1..].iter() {
            let transformed_vertex = transform.matrix() * vertex;
            min.x = min.x.min(transformed_vertex.x);
            min.y = min.y.min(transformed_vertex.y);
            min.z = min.z.min(transformed_vertex.z);

            max.x = max.x.max(transformed_vertex.x);
            max.y = max.y.max(transformed_vertex.y);
            max.z = max.z.max(transformed_vertex.z);
        }

        BoundingVolume::from_bounds(min, max)
    }

    pub fn merge(a: &BoundingVolume, b: &BoundingVolume) -> BoundingVolume {
        let mut min = a.bounds_min;
        let mut max = a.bounds_max;
        min.x = min.x.min(b.bounds_min.x);
        min.y = min.y.min(b.bounds_min.y);
        min.z = min.z.min(b.bounds_min.z);

        max.x = max.x.max(b.bounds_max.x);
        max.y = max.y.max(b.bounds_max.y);
        max.z = max.z.max(b.bounds_max.z);

        BoundingVolume::from_bounds(min, max)
    }

    pub fn intersect(&self, ray: &Ray) -> bool {
        let translated_center = self.center - ray.origin;
        let half = (self.bounds_max - self.bounds_min) / 2.0;
        let half = half.component_mul(&ray.direction.map(|c| c.signum()));

        let d0 = (translated_center.x - half.x) / ray.direction.x;
        let d1 = (translated_center.x + half.x) / ray.direction.x;
        let dy_min = (translated_center.y - half.y) / ray.direction.y;
        let dy_max = (translated_center.y + half.y) / ray.direction.y;

        if dy_max < d0 || d1 < dy_min {
            return false;
        }

        let d0 = if dy_min > d0 { dy_min } else { d0 };
        let d1 = if d1 > dy_max { dy_max } else { d1 };

        let dz_min = (translated_center.z - half.z) / ray.direction.z;
        let dz_max = (translated_center.z + half.z) / ray.direction.z;

        if dz_max < d0 || d1 < dz_min {
            return false;
        }

        let d0 = if dz_min > d0 { dz_min } else { d0 };
        let d1 = if d1 > dz_max { dz_max } else { d1 };

        if d0 < 0.0 && d1 < 0.0 {
            return false;
        }

        true
    }
}

#[derive(Debug)]
pub enum BoundedObject {
    Unbounded(Box<dyn RaytracingObject>),
    Bounded(Box<dyn RaytracingObject>, BoundingVolume),
}

impl Intersectable for BoundedObject {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        if let Self::Bounded(_, bounding_volume) = self {
            if !bounding_volume.intersect(ray) {
                return None;
            }
        }

        let object = match self {
            Self::Unbounded(object) | Self::Bounded(object, _) => object,
        };

        let ray = &ray.transform(object.get_transform().inverse());
        object.intersect(ray)
    }
}

#[derive(Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug)]
pub struct KdTreeAccelerator {
    bounded_objects: Vec<BoundedObject>,
    unbounded_objects: Vec<BoundedObject>,
    tree: KdTree,
}

impl KdTreeAccelerator {
    pub fn new(objects: Vec<Box<dyn RaytracingObject>>) -> Self {
        let objects: Vec<BoundedObject> = objects
            .into_iter()
            .filter_map(|object| object.into_bounded_object())
            .collect();

        let (bounded_objects, unbounded_objects): (Vec<BoundedObject>, Vec<BoundedObject>) =
            objects.into_iter().partition(|object| match object {
                BoundedObject::Bounded(_, _) => true,
                BoundedObject::Unbounded(_) => false,
            });
        let indexes = bounded_objects
            .iter()
            .enumerate()
            .map(|(index, _)| index)
            .collect();

        let max_depth = (8.0 + 1.3 * (bounded_objects.len() as f64).log2()) as u8;
        let tree = KdTree::build(
            build_bounding_volume(&bounded_objects),
            &bounded_objects,
            indexes,
            max_depth,
            KdTreeConstructionOptions::default(),
        )
        .unwrap_or_else(|| KdTree::Leaf(Vec::new()));
        println!("{:?}", tree);

        Self {
            bounded_objects,
            unbounded_objects,
            tree,
        }
    }

    pub fn raycast(&self, ray: &Ray) -> Option<Intersection> {
        self.raycast_int(&self.tree, ray)
    }

    pub fn shadow_cast(&self, ray: &Ray, max_distance: f64) -> bool {
        self.shadow_cast_int(&self.tree, ray, max_distance)
    }

    fn raycast_int(&self, tree: &KdTree, ray: &Ray) -> Option<Intersection> {
        match tree {
            KdTree::Node {
                bounding_volume,
                left,
                right,
                ..
            } => {
                if !bounding_volume.intersect(ray) {
                    None
                } else {
                    self.raycast_int(left, ray)
                        .into_iter()
                        .chain(self.raycast_int(right, ray).into_iter())
                        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal))
                }
            }
            KdTree::Leaf(object_indexes) => object_indexes
                .iter()
                .filter_map(|index| self.bounded_objects[*index].intersect(&ray))
                .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal)),
        }
    }

    fn shadow_cast_int(&self, tree: &KdTree, ray: &Ray, max_distance: f64) -> bool {
        match tree {
            KdTree::Node {
                bounding_volume,
                left,
                right,
                ..
            } => {
                if !bounding_volume.intersect(ray) {
                    false
                } else {
                    self.shadow_cast_int(left, ray, max_distance)
                        || self.shadow_cast_int(right, ray, max_distance)
                }
            }
            KdTree::Leaf(object_indexes) => object_indexes
                .iter()
                .filter_map(|index| self.bounded_objects[*index].intersect(&ray))
                .any(|intersection| intersection.distance <= max_distance),
        }
    }
}

#[derive(Copy, Clone)]
struct KdTreeConstructionOptions {
    max_objects: usize,
    intersection_cost: u8,
    traversal_cost: u8,
    empty_bonus: f64,
}

impl Default for KdTreeConstructionOptions {
    fn default() -> Self {
        Self {
            max_objects: 2,
            intersection_cost: 80,
            traversal_cost: 1,
            empty_bonus: 0.0,
        }
    }
}

pub enum KdTree {
    Node {
        split_axis: Axis,
        split_location: f64,
        bounding_volume: BoundingVolume,

        left: Box<KdTree>,
        right: Box<KdTree>,
    },
    Leaf(Vec<usize>),
}

impl fmt::Debug for KdTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node {
                split_axis,
                split_location,
                left,
                right,
                ..
            } => write!(
                f,
                "Node {{ split_axis: {:?}, split_location: {:?}, left: {:?}, right: {:?} }}",
                split_axis, split_location, left, right,
            ),
            Self::Leaf(indexes) => write!(f, "Leaf({:?})", indexes),
        }
    }
}

impl KdTree {
    fn build(
        bounds: BoundingVolume,
        objects: &[BoundedObject],
        indexes: Vec<usize>,
        max_depth: u8,
        options: KdTreeConstructionOptions,
    ) -> Option<Self> {
        if indexes.is_empty() {
            return None;
        } else if indexes.len() <= options.max_objects || max_depth == 0 {
            return Some(Self::Leaf(indexes));
        }

        let mut left = Vec::new();
        let mut right = Vec::new();
        for index in &indexes {
            if rand::random() {
                left.push(*index);
            } else {
                right.push(*index);
            }
        }

        let left = Self::build(bounds, objects, left, max_depth - 1, options);
        let right = Self::build(bounds, objects, right, max_depth - 1, options);

        match (left, right) {
            (Some(left), Some(right)) => {
                let bounding_volume = build_bounding_volume(objects);

                Some(Self::Node {
                    split_axis: Axis::X,
                    split_location: 0.0,
                    bounding_volume,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            (None, Some(leaf)) | (Some(leaf), None) => Some(leaf),
            (None, None) => None,
        }
    }
}
