use derive_builder::Builder;
use nalgebra::Vector3;
use num_traits::identities::Zero;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MaterialSide {
    Front,
    Back,
}

impl Default for MaterialSide {
    fn default() -> Self {
        MaterialSide::Front
    }
}

#[derive(Builder, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[builder(default)]
#[serde(default)]
pub struct PhongMaterial {
    pub side: MaterialSide,
    pub color: Vector3<f64>,
    pub emissive: Vector3<f64>,
    pub specular: Vector3<f64>,
    pub shininess: f64,
}

impl Default for PhongMaterial {
    fn default() -> Self {
        Self {
            side: MaterialSide::default(),
            color: Vector3::zero(),
            emissive: Vector3::zero(),
            specular: Vector3::zero(),
            shininess: 30.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all(deserialize = "lowercase"))]
pub enum Material {
    Phong(PhongMaterial),
}

impl Default for Material {
    fn default() -> Self {
        Material::Phong(PhongMaterial::default())
    }
}

impl Material {
    pub fn side(&self) -> MaterialSide {
        match self {
            Material::Phong(material) => material.side,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn it_deserializes_defaults() {
        assert_eq!(
            serde_json::from_value::<Material>(json!({ "type": "phong" })).unwrap(),
            Material::Phong(PhongMaterial::default())
        );
    }
}
