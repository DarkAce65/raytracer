use super::{Intersectable, Loadable};
use crate::core::{Bounds, Material, MaterialSide, Transform, Transformed};
use crate::object3d::Object3D;
use crate::ray_intersection::{Intersection, Ray, RayType};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use serde::Deserialize;
use std::f64::EPSILON;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plane {
    #[serde(default)]
    transform: Transform,
    normal: Unit<Vector3<f64>>,
    material: Material,
    children: Option<Vec<Object3D>>,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            normal: Vector3::y_axis(),
            material: Material::default(),
            children: None,
        }
    }
}

impl Loadable for Plane {}

impl Transformed for Plane {
    fn get_transform(&self) -> Transform {
        self.transform
    }
}

impl Intersectable for Plane {
    fn make_bounding_volume(&self) -> Bounds {
        Bounds::Unbounded
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

    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let ray = &ray.transform(self.get_transform().inverse());

        let n_dot_v = self.normal.dot(&-ray.direction);

        if match (self.material.side(), ray.ray_type) {
            (MaterialSide::Both, _) | (_, RayType::Shadow) => n_dot_v.abs() < EPSILON,
            (MaterialSide::Front, _) => n_dot_v < EPSILON,
            (MaterialSide::Back, _) => -n_dot_v < EPSILON,
        } {
            return None;
        }

        let distance = ray.origin.coords.dot(&self.normal) / n_dot_v;
        if distance >= 0.0 {
            return Some(Intersection {
                distance,
                object: self,
            });
        }

        None
    }

    fn surface_normal(&self, _hit_point: &Point3<f64>) -> Unit<Vector3<f64>> {
        self.normal
    }

    fn uv(&self, _hit_point: &Point3<f64>, _normal: &Unit<Vector3<f64>>) -> Vector2<f64> {
        Vector2::new(0.0, 0.0)
    }
}
