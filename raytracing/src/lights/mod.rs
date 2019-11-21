mod point;

use crate::core::Object3D;
use std::fmt::Debug;
use std::marker::{Send, Sync};

pub use point::*;

pub trait Light: Send + Sync + Debug + Object3D {}

impl Light for PointLight {}
