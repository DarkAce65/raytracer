use super::raytracing_scene::RaytracingScene;
use super::{Camera, RenderOptions};
use crate::core::{KdTreeAccelerator, Texture, Transform};
use crate::lights::Light;
use crate::primitives::Object3D;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Scene {
    #[serde(flatten)]
    pub render_options: RenderOptions,
    loaded: bool,
    camera: Camera,
    lights: Vec<Light>,
    objects: Vec<Object3D>,

    #[serde(skip)]
    textures: HashMap<String, Texture>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            render_options: RenderOptions::default(),
            loaded: false,
            camera: Camera::default(),
            lights: Vec::new(),
            objects: Vec::new(),

            textures: HashMap::new(),
        }
    }
}

impl Scene {
    pub fn new(render_options: RenderOptions, camera: Camera) -> Self {
        Self {
            render_options,
            camera,
            ..Scene::default()
        }
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    /// # Panics
    ///
    /// Will panic if scene assets have been loaded already
    pub fn add_object(&mut self, object: Object3D) {
        if self.loaded {
            panic!("objects cannot be added after scene assets have loaded");
        }

        self.objects.push(object);
    }

    /// # Panics
    ///
    /// Will panic if scene assets have been loaded already
    pub fn load_assets(&mut self, asset_base: &Path) {
        if self.loaded {
            panic!("assets are already loaded for scene");
        }

        for object in &mut self.objects {
            Object3D::load_assets(object, asset_base, &mut self.textures);
        }
        self.loaded = true;
    }

    pub fn build_raytracing_scene(self) -> RaytracingScene {
        RaytracingScene::from_scene(self)
    }
}

impl RaytracingScene {
    fn from_scene(scene: Scene) -> Self {
        let root_transform = Transform::default();
        let mut objects = Vec::new();
        for object in scene.objects {
            objects.append(&mut object.flatten_to_world(&root_transform));
        }
        let object_tree = KdTreeAccelerator::new(objects);

        RaytracingScene::new(
            scene.render_options,
            scene.camera.into(),
            scene.lights,
            scene.textures,
            object_tree,
        )
    }
}
