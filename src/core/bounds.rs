use super::Transform;
use crate::primitives::RaytracingObject;
use crate::ray_intersection::{Intersectable, Intersection, Ray};
use nalgebra::Point3;
use std::cmp::Ordering::{self, Equal};
use std::fmt;

fn build_bounding_volume(bounding_volumes: &[BoundingVolume]) -> BoundingVolume {
    if bounding_volumes.is_empty() {
        panic!("trying to build a bounding volume out of nothing")
    }

    bounding_volumes[1..]
        .iter()
        .fold(bounding_volumes[0], |acc, bounding_volume| {
            BoundingVolume::merge(&acc, bounding_volume)
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

    pub fn maximum_extent(&self) -> Axis {
        let dx = self.bounds_max.x - self.bounds_min.x;
        let dy = self.bounds_max.y - self.bounds_min.y;
        let dz = self.bounds_max.z - self.bounds_min.z;

        if dx >= dy {
            if dx >= dz {
                Axis::X
            } else {
                Axis::Z
            }
        } else if dy >= dz {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    pub fn surface_area(&self) -> f64 {
        let dx = self.bounds_max.x - self.bounds_min.x;
        let dy = self.bounds_max.y - self.bounds_min.y;
        let dz = self.bounds_max.z - self.bounds_min.z;

        2.0 * (dx * dy + dy * dz + dx * dz)
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
pub struct UnboundedObject(Box<dyn RaytracingObject>);

impl Intersectable for UnboundedObject {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let object = &self.0;
        let ray = &ray.transform(object.get_transform().inverse());
        object.intersect(ray)
    }
}

#[derive(Debug)]
pub struct BoundedObject {
    object: Box<dyn RaytracingObject>,
    bounding_volume: BoundingVolume,
}

impl Intersectable for BoundedObject {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        if !self.bounding_volume.intersect(ray) {
            return None;
        }

        let ray = &ray.transform(self.object.get_transform().inverse());
        self.object.intersect(ray)
    }
}

#[derive(Debug)]
pub enum ObjectWithBounds {
    Unbounded(UnboundedObject),
    Bounded(BoundedObject),
}

impl ObjectWithBounds {
    pub fn unbounded(object: Box<dyn RaytracingObject>) -> Self {
        Self::Unbounded(UnboundedObject(object))
    }

    pub fn bounded(object: Box<dyn RaytracingObject>, bounding_volume: BoundingVolume) -> Self {
        Self::Bounded(BoundedObject {
            object,
            bounding_volume,
        })
    }
}

impl Intersectable for ObjectWithBounds {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            Self::Unbounded(object) => object.intersect(ray),
            Self::Bounded(object) => object.intersect(ray),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn iter(initial_axis: Axis) -> impl Iterator<Item = usize> {
        let start: usize = initial_axis.into();

        (start..(start + 3)).map(|a| a % 3)
    }
}

impl From<Axis> for usize {
    fn from(axis: Axis) -> Self {
        match axis {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

impl From<usize> for Axis {
    fn from(axis: usize) -> Self {
        match axis {
            0 => Axis::X,
            1 => Axis::Y,
            2 => Axis::Z,
            _ => panic!("{:?} is not a valid 3D axis", axis),
        }
    }
}

enum SplitCandidate {
    Start(f64, usize),
    End(f64, usize),
}

impl fmt::Debug for SplitCandidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start(split, _) => write!(f, "Start({:.2})", split),
            Self::End(split, _) => write!(f, "End({:.2})", split),
        }
    }
}

impl SplitCandidate {
    fn cmp(a: &SplitCandidate, b: &SplitCandidate) -> Ordering {
        let split = a.get_split().partial_cmp(&b.get_split()).unwrap_or(Equal);

        if split == Equal {
            match (a, b) {
                (SplitCandidate::Start(_, _), SplitCandidate::End(_, _)) => Ordering::Less,
                (SplitCandidate::End(_, _), SplitCandidate::Start(_, _)) => Ordering::Greater,
                _ => Equal,
            }
        } else {
            split
        }
    }

    fn get_split(&self) -> f64 {
        match self {
            SplitCandidate::Start(split, _) | SplitCandidate::End(split, _) => *split,
        }
    }
}

#[derive(Debug)]
pub struct KdTreeAccelerator {
    unbounded_objects: Vec<UnboundedObject>,
    bounded_objects: Vec<BoundedObject>,
    tree: KdTree,
}

impl KdTreeAccelerator {
    pub fn new(objects: Vec<Box<dyn RaytracingObject>>) -> Self {
        let (unbounded_objects, bounded_objects): (Vec<ObjectWithBounds>, Vec<ObjectWithBounds>) =
            objects
                .into_iter()
                .map(|object| object.into_bounded_object())
                .partition(|object| match object {
                    ObjectWithBounds::Unbounded(_) => true,
                    ObjectWithBounds::Bounded(_) => false,
                });

        let unbounded_objects: Vec<UnboundedObject> = unbounded_objects
            .into_iter()
            .map(|object| match object {
                ObjectWithBounds::Unbounded(object) => object,
                ObjectWithBounds::Bounded(_) => unreachable!(),
            })
            .collect();
        let bounded_objects: Vec<BoundedObject> = bounded_objects
            .into_iter()
            .map(|object| match object {
                ObjectWithBounds::Unbounded(_) => unreachable!(),
                ObjectWithBounds::Bounded(object) => object,
            })
            .collect();

        let (tree, bounded_objects) = if bounded_objects.is_empty() {
            (KdTree::Leaf(Vec::new()), bounded_objects)
        } else {
            let indexes = (0..bounded_objects.len()).collect();
            let max_depth = (8.0 + 1.3 * (bounded_objects.len() as f64).log2()) as u8;
            let max_bad_refines = 3;

            let bounding_volumes: Vec<BoundingVolume> = bounded_objects
                .iter()
                .map(|object| object.bounding_volume)
                .collect();

            (
                KdTree::build(
                    &bounded_objects,
                    KdTreeConstructionOptions::default(),
                    max_depth,
                    max_bad_refines,
                    build_bounding_volume(&bounding_volumes),
                    indexes,
                )
                .unwrap_or_else(|| KdTree::Leaf(Vec::new())),
                bounded_objects,
            )
        };

        Self {
            unbounded_objects,
            bounded_objects,
            tree,
        }
    }

    pub fn get_num_objects(&self) -> usize {
        self.unbounded_objects.len() + self.bounded_objects.len()
    }

    pub fn raycast(&self, ray: &Ray) -> Option<Intersection> {
        self.unbounded_objects
            .iter()
            .filter_map(|object| object.intersect(ray))
            .chain(self.raycast_tree(&self.tree, ray))
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal))
    }

    pub fn shadow_cast(&self, ray: &Ray, max_distance: f64) -> bool {
        self.unbounded_objects
            .iter()
            .filter_map(|object| object.intersect(ray))
            .any(|intersection| intersection.distance <= max_distance)
            || self.shadow_cast_tree(&self.tree, ray, max_distance)
    }

    fn raycast_tree(&self, tree: &KdTree, ray: &Ray) -> Option<Intersection> {
        match tree {
            KdTree::Node {
                bounding_volume,
                left,
                right,
                ..
            } => {
                if bounding_volume.intersect(ray) {
                    self.raycast_tree(left, ray)
                        .into_iter()
                        .chain(self.raycast_tree(right, ray).into_iter())
                        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal))
                } else {
                    None
                }
            }
            KdTree::Leaf(object_indexes) => object_indexes
                .iter()
                .filter_map(|index| self.bounded_objects[*index].intersect(&ray))
                .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal)),
        }
    }

    fn shadow_cast_tree(&self, tree: &KdTree, ray: &Ray, max_distance: f64) -> bool {
        match tree {
            KdTree::Node {
                bounding_volume,
                left,
                right,
                ..
            } => {
                if bounding_volume.intersect(ray) {
                    self.shadow_cast_tree(left, ray, max_distance)
                        || self.shadow_cast_tree(right, ray, max_distance)
                } else {
                    false
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
    intersection_cost: f64,
    traversal_cost: f64,
    empty_bonus: f64,
}

impl Default for KdTreeConstructionOptions {
    fn default() -> Self {
        Self {
            max_objects: 2,
            intersection_cost: 80.0,
            traversal_cost: 1.0,
            empty_bonus: 0.5,
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
        objects: &[BoundedObject],
        options: KdTreeConstructionOptions,
        max_depth: u8,
        max_bad_refines: u8,
        bounding_volume: BoundingVolume,
        indexes: Vec<usize>,
    ) -> Option<Self> {
        if indexes.is_empty() {
            return None;
        } else if indexes.len() <= options.max_objects || max_depth == 0 {
            return Some(Self::Leaf(indexes));
        }

        let split_axis = bounding_volume.maximum_extent();
        let total_surface_area = bounding_volume.surface_area();
        let bounds_diagonal = bounding_volume.bounds_max - bounding_volume.bounds_min;
        let old_cost = options.intersection_cost * indexes.len() as f64;

        let mut max_bad_refines = max_bad_refines;
        let mut split_attempts = 0;

        let mut split_candidates = Vec::new();
        let mut best_axis_and_split = None;
        let mut best_cost = f64::INFINITY;

        for axis in Axis::iter(split_axis) {
            split_candidates.clear();
            for &index in &indexes {
                let object_bounds = objects[index].bounding_volume;
                split_candidates.push(SplitCandidate::Start(object_bounds.bounds_min[axis], index));
                split_candidates.push(SplitCandidate::End(object_bounds.bounds_max[axis], index));
            }
            split_candidates.sort_by(|a, b| SplitCandidate::cmp(a, b));

            let mut below = 0;
            let mut above = indexes.len();
            for (index, split_candidate) in split_candidates.iter().enumerate() {
                if let SplitCandidate::End(_, _) = split_candidate {
                    above -= 1;
                }

                let split = split_candidate.get_split();

                if bounding_volume.bounds_min[axis] < split
                    && split < bounding_volume.bounds_max[axis]
                {
                    let other_axis0 = (axis + 1) % 3;
                    let other_axis1 = (axis + 2) % 3;
                    let d = bounds_diagonal[other_axis0] * bounds_diagonal[other_axis1];
                    let surface_area_below = 2.0
                        * (d + (split - bounding_volume.bounds_min[axis])
                            * (bounds_diagonal[other_axis0] + bounds_diagonal[other_axis1]));
                    let surface_area_above = 2.0
                        * (d + (bounding_volume.bounds_max[axis] - split)
                            * (bounds_diagonal[other_axis0] + bounds_diagonal[other_axis1]));

                    let area_below = surface_area_below / total_surface_area;
                    let area_above = surface_area_above / total_surface_area;
                    let empty_bonus = if above == 0 || below == 0 {
                        options.empty_bonus
                    } else {
                        0.0
                    };
                    let cost = options.traversal_cost
                        + options.intersection_cost
                            * (1.0 - empty_bonus)
                            * (area_below * below as f64 + area_above * above as f64);

                    if cost < best_cost {
                        best_cost = cost;
                        best_axis_and_split = Some((axis, index));
                    }
                }

                if let SplitCandidate::Start(_, _) = split_candidate {
                    below += 1;
                }
            }

            if best_axis_and_split.is_none() && split_attempts < 2 {
                split_attempts += 1;
                continue;
            }

            if best_cost > old_cost {
                max_bad_refines -= 1;
            }

            if best_axis_and_split.is_none()
                || max_bad_refines == 0
                || (best_cost > 4.0 * old_cost && indexes.len() < 16)
            {
                return Some(Self::Leaf(indexes));
            }

            break;
        }

        let (split_axis, split_index) = best_axis_and_split.unwrap();
        let split_location = split_candidates[split_index].get_split();

        let mut left = Vec::new();
        let mut right = Vec::new();
        for (index, split_candidate) in split_candidates.iter().enumerate() {
            match index.cmp(&split_index) {
                Ordering::Less => {
                    if let SplitCandidate::Start(_, object_index) = split_candidate {
                        left.push(*object_index);
                    }
                }
                Ordering::Greater => {
                    if let SplitCandidate::End(_, object_index) = split_candidate {
                        right.push(*object_index);
                    }
                }
                Ordering::Equal => {}
            }
        }

        let mut left_bound = bounding_volume.bounds_max;
        left_bound[split_axis] = split_location;
        let left_bounding_volume =
            BoundingVolume::from_bounds(bounding_volume.bounds_min, left_bound);
        let left = Self::build(
            objects,
            options,
            max_depth - 1,
            max_bad_refines,
            left_bounding_volume,
            left,
        );

        let mut right_bound = bounding_volume.bounds_min;
        right_bound[split_axis] = split_location;
        let right_bounding_volume =
            BoundingVolume::from_bounds(right_bound, bounding_volume.bounds_max);
        let right = Self::build(
            objects,
            options,
            max_depth - 1,
            max_bad_refines,
            right_bounding_volume,
            right,
        );

        match (left, right) {
            (Some(left), Some(right)) => Some(Self::Node {
                split_axis: split_axis.into(),
                split_location,
                bounding_volume,
                left: Box::new(left),
                right: Box::new(right),
            }),
            (None, Some(leaf)) | (Some(leaf), None) => Some(leaf),
            (None, None) => None,
        }
    }
}
