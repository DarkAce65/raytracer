use crate::primitives::Primitive;
use nalgebra::Vector3;
use num_traits::identities::Zero;

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

pub fn raycast(scene: &Scene, x: f32, y: f32) -> [u8; 4] {
    let ray = Ray {
        origin: Vector3::zero(),
        direction: Vector3::from([x, y, -1.0]).normalize(),
    };

    let mut color = [0; 4];

    for object in scene.objects.iter() {
        if object.intersects(&ray) {
            color = object.color();
        }
    }

    color
}
