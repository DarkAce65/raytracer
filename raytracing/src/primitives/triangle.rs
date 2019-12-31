use super::{Intersectable, Loadable};
use crate::core::{BoundingVolume, Bounds, Material, Transform, Transformed};
use crate::object3d::Object3D;
use crate::ray_intersection::{Intersection, Ray};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use serde::Deserialize;
use std::f64::EPSILON;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TriangleData {
    #[serde(default)]
    transform: Transform,
    vertices: [Point3<f64>; 3],
    material: Material,
    children: Option<Vec<Object3D>>,
}

impl Default for TriangleData {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            vertices: [Point3::origin(), Point3::origin(), Point3::origin()],
            material: Material::default(),
            children: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "TriangleData")]
pub struct Triangle {
    transform: Transform,
    vertices: [Point3<f64>; 3],
    normal: Unit<Vector3<f64>>,
    material: Material,
    children: Option<Vec<Object3D>>,
}

impl From<TriangleData> for Triangle {
    fn from(data: TriangleData) -> Self {
        Self {
            transform: data.transform,
            vertices: data.vertices,
            normal: Triangle::compute_normal(data.vertices),
            material: data.material,
            children: data.children,
        }
    }
}

impl Triangle {
    pub fn new(
        transform: Transform,
        vertices: [Point3<f64>; 3],
        material: Material,
        children: Option<Vec<Object3D>>,
    ) -> Self {
        let data = TriangleData {
            transform,
            vertices,
            material,
            children,
        };

        data.into()
    }

    fn compute_normal(vertices: [Point3<f64>; 3]) -> Unit<Vector3<f64>> {
        let edge1 = vertices[1] - vertices[0];
        let edge2 = vertices[2] - vertices[0];

        Unit::new_normalize(edge2.cross(&edge1))
    }
}

impl Loadable for Triangle {}

impl Transformed for Triangle {
    fn get_transform(&self) -> Transform {
        self.transform
    }
}

impl Intersectable for Triangle {
    fn make_bounding_volume(&self) -> Bounds {
        let mut min = self.vertices[0];
        let mut max = min;
        for vertex in self.vertices[1..].iter() {
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);

            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }

        Bounds::Bounded(BoundingVolume::from_bounds_and_transform(
            min,
            max,
            &self.transform,
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
        let normal = -self.normal;
        let denom = normal.dot(&ray.direction);
        if denom > EPSILON {
            let view = -ray.origin.coords;
            let distance = view.dot(&normal) / denom;

            if distance >= 0.0 {
                let hit_point = ray.origin + ray.direction * distance;

                let edge0 = self.vertices[1] - self.vertices[0];
                let edge1 = self.vertices[2] - self.vertices[1];
                let edge2 = self.vertices[0] - self.vertices[2];
                let c0 = hit_point - self.vertices[0];
                let c1 = hit_point - self.vertices[1];
                let c2 = hit_point - self.vertices[2];
                if normal.dot(&edge0.cross(&c0)) > 0.0
                    && normal.dot(&edge1.cross(&c1)) > 0.0
                    && normal.dot(&edge2.cross(&c2)) > 0.0
                {
                    return Some(Intersection {
                        distance,
                        object: self,
                    });
                }
            }
        }

        None
    }

    fn surface_normal(&self, _hit_point: &Point3<f64>) -> Unit<Vector3<f64>> {
        self.normal
    }

    fn uv(&self, _hit_point: &Point3<f64>, _normal: &Unit<Vector3<f64>>) -> Vector2<f64> {
        Vector2::new(0.0, 0.0)
    }
}
