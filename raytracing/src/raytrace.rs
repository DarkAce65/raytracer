use crate::primitives::Primitive;
use nalgebra::Vector3;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

pub struct Scene {
    pub objects: Vec<Box<dyn Primitive>>,
}

pub fn raycast(scene: &Scene, x: u32, y: u32) -> [u8; 4] {
    let ray = Ray {
        origin: Vector3::from([x as f32, y as f32, 0.0]),
        direction: Vector3::from([0.0, 0.0, -1.0]),
    };

    let mut color = [0; 4];

    for object in scene.objects.iter() {
        if object.intersects(&ray) {
            color = object.color();
        }
    }

    color
}
