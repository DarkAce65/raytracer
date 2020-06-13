#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::wildcard_imports
)]

mod core;
mod lights;
mod primitives;
mod ray_intersection;
mod scene;

pub use lights::{AmbientLight, Light, PointLight};
pub use primitives::{Cube, Group, Mesh, Object3D, Plane, Sphere, Triangle};
pub use scene::Scene;
