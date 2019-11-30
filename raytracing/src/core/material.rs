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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
#[serde(default)]
pub struct PhysicalMaterial {
    pub side: MaterialSide,
    pub color: Vector3<f64>,
    pub emissive: Vector3<f64>,
    pub emissive_intensity: f64,
    pub roughness: f64,
    pub metalness: f64,
    pub reflectivity: f64,
    pub refractive_index: f64,
}

impl Default for PhysicalMaterial {
    fn default() -> Self {
        Self {
            side: MaterialSide::default(),
            color: Vector3::zero(),
            emissive: Vector3::zero(),
            emissive_intensity: 0.0,
            roughness: 0.5,
            metalness: 0.0,
            reflectivity: 0.0,
            refractive_index: 1.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all(deserialize = "lowercase"))]
pub enum Material {
    Phong(PhongMaterial),
    Physical(PhysicalMaterial),
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
            Material::Physical(material) => material.side,
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
        assert_eq!(
            serde_json::from_value::<Material>(json!({ "type": "physical" })).unwrap(),
            Material::Physical(PhysicalMaterial::default())
        );
    }

    #[test]
    fn it_deserializes_with_parameters() {
        assert_eq!(
            serde_json::from_value::<Material>(json!({
                "type": "phong",
                "color": [1, 0.3, 0.4]
            }))
            .unwrap(),
            Material::Phong(PhongMaterial {
                color: Vector3::from([1.0, 0.3, 0.4]),
                ..Default::default()
            })
        );

        assert_eq!(
            serde_json::from_value::<Material>(json!({
                "type": "physical",
                "color": [1, 0.3, 0.4]
            }))
            .unwrap(),
            Material::Physical(PhysicalMaterial {
                color: Vector3::from([1.0, 0.3, 0.4]),
                ..Default::default()
            })
        );
    }
}
