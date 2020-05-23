use super::{HasMaterial, Object3D, Primitive, RaytracingObject};
use crate::core::{
    quadratic, BoundedObject, BoundingVolume, Material, MaterialSide, Transform, Transformed,
};
use crate::ray_intersection::{IntermediateData, Intersectable, Intersection, Ray, RayType};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use serde::Deserialize;
use std::f64::consts::FRAC_1_PI;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Sphere {
    radius: f64,
    transform: Transform,
    pub material: Material,

    pub children: Option<Vec<Object3D>>,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            radius: 1.0,
            material: Material::default(),

            children: None,
        }
    }
}

impl Sphere {
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

        objects.push(Box::new(RaytracingSphere::new(
            self.radius,
            transform,
            self.material,
        )));

        objects
    }
}

#[derive(Debug)]
pub struct RaytracingSphere {
    radius: f64,
    world_transform: Transform,
    material: Material,
}

impl RaytracingSphere {
    pub fn new(radius: f64, world_transform: Transform, material: Material) -> Self {
        Self {
            radius,
            world_transform,
            material,
        }
    }
}

impl HasMaterial for RaytracingSphere {
    fn get_material(&self) -> &Material {
        &self.material
    }
}

impl Transformed for RaytracingSphere {
    fn get_transform(&self) -> &Transform {
        &self.world_transform
    }
}

impl Intersectable for RaytracingSphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let hypot = ray.origin.coords;
        let ray_proj = hypot.dot(&ray.direction);
        let a = ray.direction.magnitude_squared();
        let b = 2.0 * ray_proj;
        let c = hypot.magnitude_squared() - self.radius * self.radius;

        if let Some((t0, t1)) = quadratic(a, b, c) {
            debug_assert!(t0 <= t1);

            let t = match (self.material.side(), ray.ray_type) {
                (MaterialSide::Both, _) | (_, RayType::Shadow) => {
                    if t0 < 0.0 {
                        t1
                    } else {
                        t0
                    }
                }
                (MaterialSide::Front, _) => t0,
                (MaterialSide::Back, _) => t1,
            };
            if t < 0.0 {
                return None;
            }

            return Some(Intersection::new(self, t));
        }

        None
    }
}

impl Primitive for RaytracingSphere {
    fn into_bounded_object(self: Box<Self>) -> Option<BoundedObject> {
        Some(BoundedObject::bounded(
            BoundingVolume::from_bounds_and_transform(
                Point3::from([-self.radius; 3]),
                Point3::from([self.radius; 3]),
                self.get_transform(),
            ),
            self,
        ))
    }

    fn surface_normal(
        &self,
        object_hit_point: &Point3<f64>,
        _intermediate: IntermediateData,
    ) -> Unit<Vector3<f64>> {
        Unit::new_normalize(object_hit_point.coords)
    }

    fn uv(
        &self,
        object_hit_point: &Point3<f64>,
        _object_normal: &Unit<Vector3<f64>>,
        _intermediate: IntermediateData,
    ) -> Vector2<f64> {
        let hit_point = object_hit_point.coords.map(|c| c / self.radius / 2.0);

        Vector2::new(
            0.5 - hit_point.z.atan2(hit_point.x) * FRAC_1_PI / 2.0,
            0.5 + (2.0 * hit_point.y).asin() * FRAC_1_PI,
        )
    }
}
