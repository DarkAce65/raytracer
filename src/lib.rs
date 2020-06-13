#![deny(clippy::all)]

mod core;
mod lights;
mod primitives;
mod ray_intersection;
mod scene;

pub use lights::{AmbientLight, Light, PointLight};
pub use primitives::{Cube, Group, Mesh, Object3D, Plane, Sphere, Triangle};
pub use scene::Scene;
