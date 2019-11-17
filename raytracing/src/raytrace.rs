use crate::primitives::{Intersection, Primitive};
use nalgebra::{Vector3, Vector4};
use num_traits::identities::Zero;
use std::cmp::Ordering::Equal;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f32,
    pub objects: Vec<Box<dyn Primitive>>,
}

pub fn raycast(scene: &Scene, x: f32, y: f32) -> Vector4<f32> {
    let ray = Ray {
        origin: Vector3::zero(),
        direction: Vector3::from([x, y, -1.0]).normalize(),
    };

    let intersection: Option<Intersection> = scene
        .objects
        .iter()
        .rev()
        .filter_map(|object| object.intersect(&ray))
        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal));

    if let Some(intersection) = intersection {
        return intersection
            .normal
            .insert_row(3, intersection.object.color().w);
        // return intersection.object.color().xyz().component_mul (&-intersection.normal).insert_row(3,intersection.object.color().w);
    }

    Vector4::zero()
}
