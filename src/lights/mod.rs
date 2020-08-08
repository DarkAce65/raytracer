mod ambient;
mod point;

use serde::Deserialize;
use std::fmt::Debug;

pub use ambient::AmbientLight;
pub use point::PointLight;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Light {
    Ambient(AmbientLight),
    Point(Box<PointLight>),
}
