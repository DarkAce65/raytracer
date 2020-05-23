use super::{HasMaterial, Loadable, Object3D, Primitive, SemanticObject};
use crate::core::{BoundedObject, Material, MaterialSide, Transform, Transformed};
use crate::ray_intersection::{IntermediateData, Intersectable, Intersection, Ray, RayType};
use nalgebra::{Point3, Rotation3, Unit, Vector2, Vector3};
use serde::Deserialize;
use std::f64::EPSILON;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SemanticPlane {
    normal: Unit<Vector3<f64>>,
    transform: Transform,
    pub material: Material,

    pub children: Option<Vec<SemanticObject>>,
}

impl Default for SemanticPlane {
    fn default() -> Self {
        Self {
            normal: Vector3::y_axis(),
            transform: Transform::default(),
            material: Material::default(),

            children: None,
        }
    }
}

impl SemanticPlane {
    pub fn flatten_to_world(self, transform: &Transform) -> Vec<Box<dyn Object3D>> {
        let transform = transform * self.transform;

        let mut objects: Vec<Box<dyn Object3D>> = Vec::new();

        if let Some(children) = self.children {
            for child in children {
                let child_objects: Vec<Box<dyn Object3D>> = child.flatten_to_world(&transform);
                objects.extend(child_objects);
            }
        }

        objects.push(Box::new(Plane::new(self.normal, transform, self.material)));

        objects
    }
}

#[derive(Debug)]
pub struct Plane {
    normal: Unit<Vector3<f64>>,
    transform: Transform,
    material: Material,
}

impl Plane {
    pub fn new(normal: Unit<Vector3<f64>>, transform: Transform, material: Material) -> Self {
        Self {
            normal,
            transform,
            material,
        }
    }
}

impl HasMaterial for Plane {
    fn get_material(&self) -> &Material {
        &self.material
    }

    fn get_material_mut(&mut self) -> &mut Material {
        &mut self.material
    }
}

impl Loadable for Plane {}

impl Transformed for Plane {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
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
            return Some(Intersection::new(self, distance));
        }

        None
    }
}

impl Primitive for Plane {
    fn into_bounded_object(self: Box<Self>) -> Option<BoundedObject> {
        Some(BoundedObject::unbounded(self))
    }

    fn surface_normal(
        &self,
        _object_hit_point: &Point3<f64>,
        _intermediate: IntermediateData,
    ) -> Unit<Vector3<f64>> {
        self.normal
    }

    fn uv(
        &self,
        object_hit_point: &Point3<f64>,
        object_normal: &Unit<Vector3<f64>>,
        _intermediate: IntermediateData,
    ) -> Vector2<f64> {
        let p = Rotation3::rotation_between(&object_normal, &Vector3::y_axis()).unwrap()
            * object_hit_point;

        Vector2::new(p.x, p.z)
    }
}
