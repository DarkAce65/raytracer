use super::Transform;
use crate::primitives::Object3D;
use crate::ray_intersection::{Intersectable, Intersection, Ray};
use nalgebra::Point3;
use std::cmp::Ordering::Equal;

#[derive(Copy, Clone, Debug)]
pub struct BoundingVolume {
    center: Point3<f64>,
    bounds_min: Point3<f64>,
    bounds_max: Point3<f64>,
}

impl BoundingVolume {
    pub fn from_bounds(bounds_min: Point3<f64>, bounds_max: Point3<f64>) -> Self {
        assert!(bounds_max >= bounds_min);

        Self {
            center: nalgebra::center(&bounds_min, &bounds_max),
            bounds_min,
            bounds_max,
        }
    }

    pub fn from_bounds_and_transform(
        bounds_min: Point3<f64>,
        bounds_max: Point3<f64>,
        transform: &Transform,
    ) -> Self {
        assert!(bounds_max >= bounds_min);

        let mut vertices = [Point3::origin(); 8];
        let mut i = 0;
        for x in &[bounds_min.x, bounds_max.x] {
            for y in &[bounds_min.y, bounds_max.y] {
                for z in &[bounds_min.z, bounds_max.z] {
                    vertices[i] = Point3::new(*x, *y, *z);
                    i += 1;
                }
            }
        }

        let mut min = transform.matrix() * vertices[0];
        let mut max = min;
        for vertex in vertices[1..].iter() {
            let transformed_vertex = transform.matrix() * vertex;
            min.x = min.x.min(transformed_vertex.x);
            min.y = min.y.min(transformed_vertex.y);
            min.z = min.z.min(transformed_vertex.z);

            max.x = max.x.max(transformed_vertex.x);
            max.y = max.y.max(transformed_vertex.y);
            max.z = max.z.max(transformed_vertex.z);
        }

        BoundingVolume::from_bounds(min, max)
    }

    pub fn merge(a: Option<BoundingVolume>, b: Option<BoundingVolume>) -> Option<BoundingVolume> {
        if a.is_none() || b.is_none() {
            return None;
        }

        let a = a.unwrap();
        let b = b.unwrap();

        let mut min = a.bounds_min;
        let mut max = a.bounds_max;
        min.x = min.x.min(b.bounds_min.x);
        min.y = min.y.min(b.bounds_min.y);
        min.z = min.z.min(b.bounds_min.z);

        max.x = max.x.max(b.bounds_max.x);
        max.y = max.y.max(b.bounds_max.y);
        max.z = max.z.max(b.bounds_max.z);

        Some(BoundingVolume::from_bounds(min, max))
    }

    pub fn intersect(&self, ray: &Ray) -> bool {
        let ray_sign = ray.direction.map(|c| c.signum());
        let translated_center = self.center - ray.origin;
        let half = (self.bounds_max - self.bounds_min) / 2.0;

        let d0 = (translated_center.x - ray_sign.x * half.x) / ray.direction.x;
        let d1 = (translated_center.x + ray_sign.x * half.x) / ray.direction.x;
        let dy_min = (translated_center.y - ray_sign.y * half.y) / ray.direction.y;
        let dy_max = (translated_center.y + ray_sign.y * half.y) / ray.direction.y;

        if dy_max < d0 || d1 < dy_min {
            return false;
        }

        let d0 = if dy_min > d0 { dy_min } else { d0 };
        let d1 = if d1 > dy_max { dy_max } else { d1 };

        let dz_min = (translated_center.z - ray_sign.z * half.z) / ray.direction.z;
        let dz_max = (translated_center.z + ray_sign.z * half.z) / ray.direction.z;

        if dz_max < d0 || d1 < dz_min {
            return false;
        }

        let d0 = if dz_min > d0 { dz_min } else { d0 };
        let d1 = if d1 > dz_max { dz_max } else { d1 };

        if d0 < 0.0 && d1 < 0.0 {
            return false;
        }

        true
    }
}

#[derive(Debug)]
pub struct BoundedObject {
    object: Box<dyn Object3D>,
    bounding_volume: Option<BoundingVolume>,
}

impl BoundedObject {
    pub fn unbounded(object: Box<dyn Object3D>) -> Self {
        Self {
            object,
            bounding_volume: None,
        }
    }

    pub fn bounded(bounding_volume: BoundingVolume, object: Box<dyn Object3D>) -> Self {
        Self {
            object,
            bounding_volume: Some(bounding_volume),
        }
    }
}

impl Intersectable for BoundedObject {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        if let Some(bounding_volume) = self.bounding_volume {
            if !bounding_volume.intersect(ray) {
                return None;
            }
        }

        let ray = &ray.transform(self.object.get_transform().inverse());

        self.object
            .intersect(ray)
            .map(|mut intersection| {
                intersection.root_transform = Some(self.object.get_transform());
                intersection
            })
            .into_iter()
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Equal))
    }
}
