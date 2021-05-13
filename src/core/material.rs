use super::Texture;
use nalgebra::{Vector2, Vector3};
use num_traits::identities::Zero;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
pub enum MaterialSide {
    Both,
    Front,
    Back,
}

impl Default for MaterialSide {
    fn default() -> Self {
        MaterialSide::Front
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PhongMaterial {
    pub side: MaterialSide,
    pub color: Vector3<f64>,
    pub emissive: Vector3<f64>,
    pub specular: Vector3<f64>,
    pub reflectivity: f64,
    pub shininess: f64,
    #[serde(rename = "texture")]
    pub texture_path: Option<String>,
}

impl Default for PhongMaterial {
    fn default() -> Self {
        Self {
            side: MaterialSide::default(),
            color: Vector3::zero(),
            emissive: Vector3::zero(),
            specular: Vector3::zero(),
            reflectivity: 0.0,
            shininess: 30.0,
            texture_path: None,
        }
    }
}

impl PhongMaterial {
    pub fn get_color(&self, uv: Vector2<f64>, textures: &HashMap<String, Texture>) -> Vector3<f64> {
        self.texture_path
            .as_ref()
            .map_or(self.color, |texture_path| {
                let texture = textures.get(texture_path).expect("texture not loaded");
                self.color.component_mul(&texture.get_color(uv))
            })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PhysicalMaterial {
    pub side: MaterialSide,
    pub color: Vector3<f64>,
    pub opacity: f64,
    pub emissive: Vector3<f64>,
    pub emissive_intensity: f64,
    pub roughness: f64,
    pub metalness: f64,
    pub refractive_index: f64,
    #[serde(rename = "texture")]
    pub texture_path: Option<String>,
}

impl Default for PhysicalMaterial {
    fn default() -> Self {
        Self {
            side: MaterialSide::default(),
            color: Vector3::zero(),
            opacity: 1.0,
            emissive: Vector3::zero(),
            emissive_intensity: 0.0,
            roughness: 0.5,
            metalness: 0.0,
            refractive_index: 1.0,
            texture_path: None,
        }
    }
}

impl PhysicalMaterial {
    pub fn get_color(&self, uv: Vector2<f64>, textures: &HashMap<String, Texture>) -> Vector3<f64> {
        self.texture_path
            .as_ref()
            .map_or(self.color, |texture_path| {
                let texture = textures.get(texture_path).expect("texture not loaded");
                self.color.component_mul(&texture.get_color(uv))
            })
    }
}

#[derive(Clone, Debug, Deserialize)]
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
    /// # Panics
    ///
    /// Will panic if texture cannot be loaded
    pub fn load_textures(&self, asset_base: &Path, textures: &mut HashMap<String, Texture>) {
        let texture_path = match self {
            Material::Phong(material) => material.texture_path.as_ref(),
            Material::Physical(material) => material.texture_path.as_ref(),
        };

        if let Some(texture_path) = texture_path {
            if !textures.contains_key(texture_path) {
                let texture_path = texture_path.to_string();
                let mut texture = Texture::new(&texture_path);
                texture.load(asset_base).unwrap_or_else(|err| {
                    panic!(
                        "failed to load texture at path \"{}\": {}",
                        texture_path, err
                    )
                });
                textures.insert(texture_path, texture);
            }
        }
    }

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

    impl PartialEq for PhongMaterial {
        fn eq(&self, other: &PhongMaterial) -> bool {
            self.side == other.side
                && self.color == other.color
                && self.emissive == other.emissive
                && self.specular == other.specular
                && self.reflectivity == other.reflectivity
                && self.shininess == other.shininess
        }
    }

    impl PartialEq for PhysicalMaterial {
        fn eq(&self, other: &PhysicalMaterial) -> bool {
            self.side == other.side
                && self.color == other.color
                && self.opacity == other.opacity
                && self.emissive == other.emissive
                && self.emissive_intensity == other.emissive_intensity
                && self.roughness == other.roughness
                && self.metalness == other.metalness
                && self.refractive_index == other.refractive_index
        }
    }

    impl PartialEq for Material {
        fn eq(&self, other: &Material) -> bool {
            match (self, other) {
                (Material::Phong(a), Material::Phong(b)) => a == b,
                (Material::Physical(a), Material::Physical(b)) => a == b,
                _ => false,
            }
        }
    }

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
                ..PhongMaterial::default()
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
                ..PhysicalMaterial::default()
            })
        );
    }
}
