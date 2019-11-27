use super::{Drawable, Intersectable, Material};
use crate::core::{Intersection, Object3D, Ray, Transform};
use derive_builder::Builder;
use nalgebra::{Point3, Unit, Vector3};

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default)]
pub struct Cube {
    transform: Transform,
    size: f64,
    material: Material,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            size: 1.0,
            material: Material::default(),
        }
    }
}

impl Object3D for Cube {
    fn transform(&self) -> Transform {
        self.transform
    }
}

impl Intersectable for Cube {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let ray_sign = ray.direction.into_inner().map(|c| c.signum());
        let translated_center = self.transform.position - ray.origin;
        let half = self.size / 2.0;

        let d0 = (translated_center.x - ray_sign.x * half) / ray.direction.x;
        let d1 = (translated_center.x + ray_sign.x * half) / ray.direction.x;
        let dy_min = (translated_center.y - ray_sign.y * half) / ray.direction.y;
        let dy_max = (translated_center.y + ray_sign.y * half) / ray.direction.y;

        if dy_max < d0 || d1 < dy_min {
            return None;
        }

        let d0 = if dy_min > d0 { dy_min } else { d0 };
        let d1 = if d1 > dy_max { dy_max } else { d1 };

        let dz_min = (translated_center.z - ray_sign.z * half) / ray.direction.z;
        let dz_max = (translated_center.z + ray_sign.z * half) / ray.direction.z;

        if dz_max < d0 || d1 < dz_min {
            return None;
        }

        let d0 = if dz_min > d0 { dz_min } else { d0 };
        let d1 = if d1 > dz_max { dz_max } else { d1 };

        let d = if d0 < 0.0 { d1 } else { d0 };
        if d < 0.0 {
            return None;
        }

        let intersection = Intersection {
            distance: d,
            object: Box::new(*self),
        };

        Some(intersection)
    }

    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>> {
        let normal = hit_point - self.transform.position;
        let normal_sign = normal.map(|c| c.signum());
        let normal = normal.map(|c| c.abs());
        if normal.x > normal.y {
            if normal.x > normal.z {
                if normal_sign.x < 0.0 {
                    -Vector3::x_axis()
                } else {
                    Vector3::x_axis()
                }
            } else if normal_sign.z < 0.0 {
                -Vector3::z_axis()
            } else {
                Vector3::z_axis()
            }
        } else if normal.y > normal.z {
            if normal_sign.y < 0.0 {
                -Vector3::y_axis()
            } else {
                Vector3::y_axis()
            }
        } else if normal_sign.z < 0.0 {
            -Vector3::z_axis()
        } else {
            Vector3::z_axis()
        }
    }
}

impl Drawable for Cube {
    fn material(&self) -> Material {
        self.material
    }
}
