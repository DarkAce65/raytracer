use super::{Intersectable, Loadable, Triangle};
use crate::core::{Bounds, Material, Transform, Transformed};
use crate::object3d::Object3D;
use crate::ray_intersection::{Intersection, Ray};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use serde::Deserialize;
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

impl Loadable for Mesh {
    fn load_assets(&mut self, asset_base: &Path) -> bool {
        let (models, _) = load_obj(&asset_base.join(&self.file)).expect("Failed to load object");

        let mut children = Vec::new();
        for model in models.iter() {
            let mesh = &model.mesh;

            let positions: Vec<f64> = mesh.positions.iter().map(|p| (*p).into()).collect();

            for f in 0..mesh.indices.len() / 3 {
                let (idx0, idx1, idx2) = (
                    mesh.indices[3 * f] as usize,
                    mesh.indices[3 * f + 1] as usize,
                    mesh.indices[3 * f + 2] as usize,
                );
                let p0 = Point3::new(
                    positions[3 * idx0],
                    positions[3 * idx0 + 1],
                    positions[3 * idx0 + 2],
                );
                let p1 = Point3::new(
                    positions[3 * idx1],
                    positions[3 * idx1 + 1],
                    positions[3 * idx1 + 2],
                );
                let p2 = Point3::new(
                    positions[3 * idx2],
                    positions[3 * idx2 + 1],
                    positions[3 * idx2 + 2],
                );

                let face = Triangle::new(self.transform, [p0, p1, p2], self.material.clone(), None);
                children.push(Object3D::new(Box::new(face)));
            }
        }

        self.children = Some(children);

        true
    }
}

impl Transformed for Mesh {
    fn get_transform(&self) -> Transform {
        self.transform
    }
}

impl Intersectable for Mesh {
    fn make_bounding_volume(&self) -> Bounds {
        Bounds::Children
    }

    fn get_material(&self) -> &Material {
        &self.material
    }

    fn get_material_mut(&mut self) -> &mut Material {
        &mut self.material
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

    fn surface_normal(&self, _hit_point: &Point3<f64>) -> Unit<Vector3<f64>> {
        unimplemented!()
    }

    fn uv(&self, _hit_point: &Point3<f64>, _normal: &Unit<Vector3<f64>>) -> Vector2<f64> {
        unimplemented!()
    }
}
