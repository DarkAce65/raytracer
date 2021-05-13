use super::{Object3D, RaytracingObject, Triangle};
use crate::core::{Material, Transform};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use num_traits::identities::Zero;
use serde::Deserialize;
use std::path::Path;
use tobj::{load_obj, LoadOptions};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mesh {
    file: String,
    #[serde(default)]
    transform: Transform,
    #[serde(default)]
    pub material: Material,

    #[serde(default)]
    pub children: Option<Vec<Object3D>>,
}

impl Mesh {
    pub fn new(file: String, transform: Transform, material: Material) -> Self {
        Self {
            file,
            transform,
            material,
            children: None,
        }
    }

    pub fn add_child(&mut self, object: Object3D) {
        if let Some(children) = self.children.as_mut() {
            children.push(object);
        }
    }

    pub fn flatten_to_world(self, transform: &Transform) -> Vec<Box<dyn RaytracingObject>> {
        let transform = transform * self.transform;

        let mut objects: Vec<Box<dyn RaytracingObject>> = Vec::new();

        if let Some(children) = self.children {
            for child in children {
                let child_objects: Vec<Box<dyn RaytracingObject>> =
                    child.flatten_to_world(&transform);
                objects.extend(child_objects);
            }
        }

        objects
    }

    /// # Panics
    ///
    /// Will panic if object asset cannot be loaded
    pub fn load_assets(&mut self, asset_base: &Path) {
        let (models, _) = load_obj(
            &asset_base.join(&self.file),
            &LoadOptions {
                triangulate: true,
                ignore_lines: true,
                ignore_points: true,
                ..tobj::LoadOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!(
                "failed to load object at path \"{}\": {}",
                &asset_base.join(&self.file).display(),
                err
            )
        });

        let mut children: Vec<Object3D> = Vec::new();
        for model in &models {
            let mesh = &model.mesh;

            let positions: Vec<Point3<f64>> = mesh
                .positions
                .chunks_exact(3)
                .map(|position| {
                    Point3::new(
                        f64::from(position[0]),
                        f64::from(position[1]),
                        f64::from(position[2]),
                    )
                })
                .collect();
            let normals: Vec<Unit<Vector3<f64>>> = mesh
                .normals
                .chunks_exact(3)
                .map(|normal| {
                    Unit::new_normalize(Vector3::new(
                        f64::from(normal[0]),
                        f64::from(normal[1]),
                        f64::from(normal[2]),
                    ))
                })
                .collect();
            let texcoords: Vec<Vector2<f64>> = mesh
                .texcoords
                .chunks_exact(2)
                .map(|texcoords| Vector2::new(f64::from(texcoords[0]), f64::from(texcoords[1])))
                .collect();

            for face_indices in mesh.indices.chunks_exact(3) {
                let (idx0, idx1, idx2) = (
                    face_indices[0] as usize,
                    face_indices[1] as usize,
                    face_indices[2] as usize,
                );

                let p0 = positions[idx0];
                let p1 = positions[idx1];
                let p2 = positions[idx2];

                let normals = if mesh.normals.is_empty() {
                    [Triangle::compute_normal([p0, p1, p2]); 3]
                } else {
                    let n0 = normals[idx0];
                    let n1 = normals[idx1];
                    let n2 = normals[idx2];

                    [n0, n1, n2]
                };

                let texcoords = if mesh.texcoords.is_empty() {
                    [Vector2::zero(); 3]
                } else {
                    let uv0 = texcoords[idx0];
                    let uv1 = texcoords[idx1];
                    let uv2 = texcoords[idx2];

                    [uv0, uv1, uv2]
                };

                let face = Triangle::new(
                    [p0, p1, p2],
                    normals,
                    texcoords,
                    Transform::default(),
                    self.material.clone(),
                );

                children.push(Object3D::Triangle(Box::new(face)));
            }
        }

        self.children = Some(children);
    }
}
