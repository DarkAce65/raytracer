use super::{Drawable, Intersectable, Intersection};
use crate::raytrace::{Object3D, Ray};
use derive_builder::Builder;
use nalgebra::{Point3, Unit, Vector3, Vector4};

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct Cube {
    size: f64,
    center: Point3<f64>,
    color: Vector4<f64>,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            size: 1.0,
            center: Point3::origin(),
            color: Vector4::from([1.0; 4]),
        }
    }
}

impl Object3D for Cube {
    fn position(&self) -> Point3<f64> {
        self.center
    }

    fn scale(&self) -> Vector3<f64> {
        unimplemented!()
    }

    fn rotation(&self) -> Vector3<f64> {
        unimplemented!()
    }
}

impl Drawable for Cube {
    fn color(&self) -> Vector4<f64> {
        self.color
    }
}

impl Intersectable for Cube {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let translated_center = self.center - ray.origin;
        let half = self.size / 2.0;

        let mut t0 = (translated_center.x - ray.direction.x.signum() * half) / ray.direction.x;
        let mut t1 = (translated_center.x + ray.direction.x.signum() * half) / ray.direction.x;
        let tymin = (translated_center.y - ray.direction.y.signum() * half) / ray.direction.y;
        let tymax = (translated_center.y + ray.direction.y.signum() * half) / ray.direction.y;

        if t0 > tymax || t1 < tymin {
            return None;
        }

        if tymin > t0 {
            t0 = tymin;
        }
        if tymax < t1 {
            t1 = tymax;
        }

        let tzmin = (translated_center.z - ray.direction.z.signum() * half) / ray.direction.z;
        let tzmax = (translated_center.z + ray.direction.z.signum() * half) / ray.direction.z;

        if t0 > tzmax || t1 < tzmin {
            return None;
        }

        if tzmin > t0 {
            t0 = tzmin;
        }
        if tzmax < t1 {
            t1 = tzmax;
        }

        let t = if t0 < 0.0 { t1 } else { t0 };
        if t < 0.0 {
            return None;
        }

        let intersection = Intersection {
            distance: t,
            object: Box::new(*self),
        };

        Some(intersection)
    }

    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>> {
        let normal = hit_point - self.center;
        if normal.x > normal.y {
            if normal.x > normal.z {
                Vector3::x_axis()
            } else {
                Vector3::z_axis()
            }
        } else if normal.y > normal.z {
            Vector3::y_axis()
        } else {
            Vector3::z_axis()
        }
    }
}
