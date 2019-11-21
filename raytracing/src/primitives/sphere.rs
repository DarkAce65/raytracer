use super::{Drawable, Intersectable, Material};
use crate::core::{quadratic, Intersection, Object3D, Ray};
use derive_builder::Builder;
use nalgebra::{Point3, Unit, Vector3};

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct Sphere {
    radius: f64,
    center: Point3<f64>,
    material: Material,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            radius: 1.0,
            center: Point3::origin(),
            material: Material::default(),
        }
    }
}

impl Object3D for Sphere {
    fn position(&self) -> Point3<f64> {
        self.center
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let hypot = ray.origin - self.center;
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
                object: Box::new(*self),
            });
        }

        None
    }

    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>> {
        Unit::new_normalize(hit_point - self.center)
    }
}

impl Drawable for Sphere {
    fn material(&self) -> Material {
        self.material
    }
}
