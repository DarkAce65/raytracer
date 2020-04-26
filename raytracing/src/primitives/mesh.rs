use super::{HasMaterial, Intersectable, Loadable, Triangle};
use crate::core::Texture;
use crate::core::{Bounds, Material, Transform, Transformed};
use crate::object3d::Object3D;
use crate::ray_intersection::{IntermediateData, Intersection, Ray};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use num_traits::identities::Zero;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tobj::load_obj;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mesh {
    #[serde(default)]
    transform: Transform,
    file: String,
    material: Material,
    children: Option<Vec<Object3D>>,
}

impl HasMaterial for Mesh {
    fn get_material(&self) -> &Material {
        &self.material
    }

    fn get_material_mut(&mut self) -> &mut Material {
        &mut self.material
    }
}

impl Loadable for Mesh {
    fn load_assets(&mut self, asset_base: &Path, textures: &mut HashMap<String, Texture>) -> bool {
        let (models, _) = load_obj(&asset_base.join(&self.file)).expect("failed to load object");

        let mut children = Vec::new();
        for model in models.iter() {
            let mesh = &model.mesh;

            let positions: Vec<Point3<f64>> = mesh
                .positions
                .chunks_exact(3)
                .map(|position| {
                    Point3::new(position[0] as f64, position[1] as f64, position[2] as f64)
                })
                .collect();
            let normals: Vec<Unit<Vector3<f64>>> = mesh
                .normals
                .chunks_exact(3)
                .map(|normal| {
                    Unit::new_normalize(Vector3::new(
                        normal[0] as f64,
                        normal[1] as f64,
                        normal[2] as f64,
                    ))
                })
                .collect();
            let texcoords: Vec<Vector2<f64>> = mesh
                .texcoords
                .chunks_exact(2)
                .map(|texcoords| Vector2::new(texcoords[0] as f64, texcoords[1] as f64))
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
                    self.transform.clone(),
                    [p0, p1, p2],
                    normals,
                    texcoords,
                    self.material.clone(),
                    None,
                );

                children.push(Object3D::new(Box::new(face)));
            }
        }

        self.children = Some(children);

        self.load_textures(asset_base, textures);

        true
    }
}

impl Transformed for Mesh {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
}

impl Intersectable for Mesh {
    fn make_bounding_volume(&self) -> Bounds {
        Bounds::Children
    }

    fn get_children(&self) -> Option<&Vec<Object3D>> {
        self.children.as_ref()
    }

    fn get_children_mut(&mut self) -> Option<&mut Vec<Object3D>> {
        self.children.as_mut()
    }

    fn intersect(&self, _ray: &Ray) -> Option<Intersection> {
        None
    }

    fn surface_normal(
        &self,
        _object_hit_point: &Point3<f64>,
        _intermediate: IntermediateData,
    ) -> Unit<Vector3<f64>> {
        unimplemented!()
    }

    fn uv(
        &self,
        _object_hit_point: &Point3<f64>,
        _object_normal: &Unit<Vector3<f64>>,
        _intermediate: IntermediateData,
    ) -> Vector2<f64> {
        unimplemented!()
    }
}
