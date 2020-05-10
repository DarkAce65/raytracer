use super::{HasMaterial, Loadable, Object3D, Primitive};
use crate::core::{
    quadratic, BoundedObject, BoundingVolume, Material, MaterialSide, Transform, Transformed,
};
use crate::ray_intersection::{IntermediateData, Intersectable, Intersection, Ray, RayType};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use serde::Deserialize;
use std::f64::consts::FRAC_1_PI;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sphere {
    #[serde(default)]
    transform: Transform,
    radius: f64,
    material: Material,
    children: Option<Vec<Box<dyn Object3D>>>,
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

impl HasMaterial for Sphere {
    fn get_material(&self) -> &Material {
        &self.material
    }

    fn get_material_mut(&mut self) -> &mut Material {
        &mut self.material
    }
}

impl Loadable for Sphere {}

impl Transformed for Sphere {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
}

impl Intersectable for Sphere {
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

impl Primitive for Sphere {
    fn into_bounded_object(self: Box<Self>, parent_transform: &Transform) -> Option<BoundedObject> {
        let transform = parent_transform * self.get_transform();
        Some(BoundedObject::bounded(
            BoundingVolume::from_bounds_and_transform(
                Point3::from([-self.radius; 3]),
                Point3::from([self.radius; 3]),
                &transform,
            ),
            transform,
            self,
        ))
    }

    fn get_children(&self) -> Option<&Vec<Box<dyn Object3D>>> {
        self.children.as_ref()
    }

    fn get_children_mut(&mut self) -> Option<&mut Vec<Box<dyn Object3D>>> {
        self.children.as_mut()
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
