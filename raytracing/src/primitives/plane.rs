use super::{HasMaterial, Object3D, Primitive, RaytracingObject};
use crate::core::{BoundedObject, Material, MaterialSide, Transform, Transformed};
use crate::ray_intersection::{IntermediateData, Intersectable, Intersection, Ray, RayType};
use nalgebra::{Point3, Rotation3, Unit, Vector2, Vector3};
use serde::Deserialize;
use std::f64::EPSILON;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Plane {
    normal: Unit<Vector3<f64>>,
    transform: Transform,
    pub material: Material,

    pub children: Option<Vec<Object3D>>,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            normal: Vector3::y_axis(),
            transform: Transform::default(),
            material: Material::default(),

            children: None,
        }
    }
}

impl Plane {
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

        objects.push(Box::new(RaytracingPlane::new(
            self.normal,
            transform,
            self.material,
        )));

        objects
    }
}

#[derive(Debug)]
pub struct RaytracingPlane {
    normal: Unit<Vector3<f64>>,
    world_transform: Transform,
    material: Material,
}

impl RaytracingPlane {
    pub fn new(normal: Unit<Vector3<f64>>, world_transform: Transform, material: Material) -> Self {
        Self {
            normal,
            world_transform,
            material,
        }
    }
}

impl HasMaterial for RaytracingPlane {
    fn get_material(&self) -> &Material {
        &self.material
    }
}

impl Transformed for RaytracingPlane {
    fn get_transform(&self) -> &Transform {
        &self.world_transform
    }
}

impl Intersectable for RaytracingPlane {
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

impl Primitive for RaytracingPlane {
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
