use super::Intersectable;
use crate::core::{quadratic, BoundingVolume, Material, Transform, Transformed};
use crate::object3d::Object3D;
use crate::ray_intersection::{Intersection, Ray};
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

    children: Option<Vec<Object3D>>,
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

impl Transformed for Sphere {
    fn get_transform(&self) -> Transform {
        self.transform
    }
}

impl Intersectable for Sphere {
    fn make_bounding_volume(&self) -> Option<BoundingVolume> {
        Some(BoundingVolume::from_bounds_and_transform(
            Point3::from([-self.radius; 3]),
            Point3::from([self.radius; 3]),
            self.transform,
        ))
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
        let hypot = ray.origin.coords;
        let ray_proj = hypot.dot(&ray.direction);
        let a = ray.direction.magnitude_squared();
        let b = 2.0 * ray_proj;
        let c = hypot.magnitude_squared() - self.radius * self.radius;

        if let Some((t0, t1)) = quadratic(a, b, c) {
            let t = if t0 < 0.0 { t1 } else { t0 };

            if t < 0.0 {
                return None;
            }

            let distance = t;
            return Some(Intersection {
                distance,
                object: self,
            });
        }

        None
    }

    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>> {
        Unit::new_normalize(hit_point.coords)
    }

    fn uv(&self, hit_point: &Point3<f64>, _normal: &Unit<Vector3<f64>>) -> Vector2<f64> {
        let hit_point = hit_point.coords.map(|c| c / self.radius / 2.0);

        Vector2::new(
            0.5 - hit_point.z.atan2(hit_point.x) * FRAC_1_PI / 2.0,
            0.5 - (2.0 * hit_point.y).asin() * FRAC_1_PI,
        )
    }
}
