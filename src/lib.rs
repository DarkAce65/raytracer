#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::too_many_lines
)]

mod core;
mod lights;
mod primitives;
mod ray_intersection;
mod render;
mod utils;

pub use crate::core::{Material, PhongMaterial, PhysicalMaterial, Transform};
pub use crate::lights::{AmbientLight, Light, PointLight};
pub use crate::primitives::{Cube, Group, Mesh, Object3D, Plane, Sphere, Triangle};
pub use crate::render::{Camera, CastStats, RenderOptions, Scene};
