mod raytracing_scene;
mod scene;

use nalgebra::{Point3, Unit, Vector3};
use serde::Deserialize;

pub use scene::Scene;

const GAMMA: f64 = 2.2;
const BIAS: f64 = 1e-10;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Camera {
    pub fov: f64,
    pub position: Point3<f64>,
    pub target: Point3<f64>,
    pub up: Unit<Vector3<f64>>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov: 65.0,
            position: Point3::from([0.0, 0.0, 1.0]),
            target: Point3::origin(),
            up: Vector3::y_axis(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub max_depth: u8,
    pub samples_per_pixel: u16,
    pub max_reflected_rays: u16,
    pub max_occlusion_rays: u16,
    pub max_occlusion_distance: f64,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            max_depth: 3,
            samples_per_pixel: 4,
            max_reflected_rays: 32,
            max_occlusion_rays: 16,
            max_occlusion_distance: 1.0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::{Material, PhongMaterial, Transform};
    use crate::lights::{AmbientLight, Light, PointLight};
    use crate::primitives::{Cube, Object3D};
    use serde_json::json;

    #[test]
    fn it_builds_a_raytracing_scene_from_an_empty_scene_json() {
        let scene_json = json!({});
        let scene: Result<Scene, serde_json::error::Error> = serde_json::from_value(scene_json);
        assert!(scene.is_ok(), "failed to deserialize scene");

        scene.unwrap().build_raytracing_scene();
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_a_scene_json() {
        let scene_json = json!({
          "max_depth": 5,
          "width": 200,
          "height": 200,
          "camera": { "position": [2, 5, 15], "target": [-1, 0, 0] },
          "lights": [
            { "type": "ambient", "color": [0.01, 0.01, 0.01] },
            {
              "type": "point",
              "transform": [{ "translate": [-8, 3, 0] }],
              "color": [0.5, 0.5, 0.5]
            }
          ],
          "objects": [
            {
              "type": "cube",
              "size": 1,
              "transform": [{ "rotate": [[0, 1, 0], 30] }, { "translate": [0, 2, 0] }],
              "material": { "type": "phong", "color": [1, 0.1, 0.1] }
            }
          ]
        });

        let scene: Result<Scene, serde_json::error::Error> = serde_json::from_value(scene_json);
        assert!(scene.is_ok(), "failed to deserialize scene");

        scene.unwrap().build_raytracing_scene();
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_an_empty_scene() {
        let scene = Scene::new(RenderOptions::default(), Camera::default());
        scene.build_raytracing_scene();
    }

    #[test]
    fn it_builds_a_raytracing_scene_from_a_scene() {
        let mut scene = Scene::new(
            RenderOptions {
                width: 200,
                height: 200,
                max_depth: 5,
                ..RenderOptions::default()
            },
            Camera::default(),
        );

        scene.add_light(Light::Ambient(AmbientLight::new(Vector3::from([
            0.01, 0.01, 0.01,
        ]))));
        scene.add_light(Light::Point(Box::new(PointLight::new(
            Vector3::from([0.5, 0.5, 0.5]),
            1.0,
            Transform::identity().translate(Vector3::from([-8.0, 3.0, 0.0])),
        ))));

        scene.add_object(Object3D::Cube(Box::new(Cube::new(
            1.0,
            Transform::identity()
                .rotate(Vector3::y_axis(), 30.0)
                .translate(Vector3::from([0.0, 2.0, 0.0])),
            Material::Phong(PhongMaterial {
                color: Vector3::from([1.0, 0.1, 0.1]),
                ..PhongMaterial::default()
            }),
        ))));

        scene.build_raytracing_scene();
    }
}
