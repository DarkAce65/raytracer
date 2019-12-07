use super::{Drawable, Intersectable};
use crate::core::{BoundingVolume, Intersection, Material, Object3D, Ray, Transform};
use nalgebra::{Point3, Unit, Vector3};
use serde::Deserialize;
use std::f64::EPSILON;

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plane {
    #[serde(default)]
    transform: Transform,
    normal: Unit<Vector3<f64>>,
    material: Material,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            normal: Vector3::y_axis(),
            material: Material::default(),
        }
    }
}

impl Object3D for Plane {
    fn transform(&self) -> Transform {
        self.transform
    }
}

impl Intersectable for Plane {
    fn make_bounding_volume(&self) -> BoundingVolume {
        BoundingVolume::new(Box::new(*self), Point3::origin(), Point3::origin())
    }

    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let ray = &ray.transform(self.transform().inverse());
        let normal = -self.normal;
        let denom = normal.dot(&ray.direction);
        if denom > EPSILON {
            let view = -ray.origin.coords;
            let distance = view.dot(&normal) / denom;

            if distance >= 0.0 {
                return Some(Intersection {
                    distance,
                    object: Box::new(*self),
                });
            }
        }

        None
    }

    fn surface_normal(&self, _: &Point3<f64>) -> Unit<Vector3<f64>> {
        self.normal
    }
}

impl Drawable for Plane {
    fn material(&self) -> Material {
        self.material
    }
}
