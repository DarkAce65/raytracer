use super::{Intersection, Ray};
use crate::primitives::Primitive;
use nalgebra::Point3;
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct BoundingVolume {
    object: Box<dyn Primitive>,
    center: Point3<f64>,
    bounds_min: Point3<f64>,
    bounds_max: Point3<f64>,
}

impl BoundingVolume {
    pub fn new(
        object: Box<dyn Primitive>,
        bounds_min: Point3<f64>,
        bounds_max: Point3<f64>,
    ) -> Self {
        Self {
            object,
            center: nalgebra::center(&bounds_min, &bounds_max),
            bounds_min,
            bounds_max,
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let ray_sign = ray.direction.map(|c| c.signum());
        let translated_center = self.center - ray.origin;
        let half = (self.bounds_max - self.bounds_min) / 2.0;

        let d0 = (translated_center.x - ray_sign.x * half.x) / ray.direction.x;
        let d1 = (translated_center.x + ray_sign.x * half.x) / ray.direction.x;
        let dy_min = (translated_center.y - ray_sign.y * half.y) / ray.direction.y;
        let dy_max = (translated_center.y + ray_sign.y * half.y) / ray.direction.y;

        if dy_max < d0 || d1 < dy_min {
            return None;
        }

        let d0 = if dy_min > d0 { dy_min } else { d0 };
        let d1 = if d1 > dy_max { dy_max } else { d1 };

        let dz_min = (translated_center.z - ray_sign.z * half.z) / ray.direction.z;
        let dz_max = (translated_center.z + ray_sign.z * half.z) / ray.direction.z;

        if dz_max < d0 || d1 < dz_min {
            return None;
        }

        let d0 = if dz_min > d0 { dz_min } else { d0 };
        let d1 = if d1 > dz_max { dz_max } else { d1 };

        if d0 < 0.0 && d1 < 0.0 {
            return None;
        }

        self.object.intersect(ray)
    }
}

impl<'de> Deserialize<'de> for BoundingVolume {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let object: Box<dyn Primitive> = Deserialize::deserialize(deserializer)?;
        Ok(object.make_bounding_volume())
    }
}
