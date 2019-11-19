mod point;

use crate::raytrace::Object3D;
use std::marker::{Send, Sync};

pub use point::*;

pub trait Light: Send + Sync + Object3D {}
impl<T> Light for T where T: Send + Sync + Object3D {}
