use super::{Drawable, Intersectable, Material};
use crate::core::{Intersection, Object3D, Ray};
use derive_builder::Builder;
use nalgebra::{Point3, Unit, Vector3};

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct Plane {
    position: Point3<f64>,
    normal: Unit<Vector3<f64>>,
    material: Material,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            normal: Vector3::y_axis(),
            material: Material::default(),
        }
    }
}

impl Object3D for Plane {
    fn position(&self) -> Point3<f64> {
        self.position
    }
}

impl Intersectable for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let normal = -self.normal;
        let denom = normal.dot(&ray.direction);
        if denom > 1e-6 {
            let view = self.position - ray.origin;
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
