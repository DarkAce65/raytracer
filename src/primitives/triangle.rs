use super::{HasMaterial, Object3D, Primitive, RaytracingObject};
use crate::core::{
    BoundingVolume, Material, MaterialSide, ObjectWithBounds, Transform, Transformed,
};
use crate::ray_intersection::{IntermediateData, Intersectable, Intersection, Ray, RayType};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use num_traits::identities::Zero;
use serde::Deserialize;
use std::f64::EPSILON;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VertexPNT {
    position: Point3<f64>,
    normal: Unit<Vector3<f64>>,
    texcoords: Vector2<f64>,
}

impl VertexPNT {
    fn new(position: Point3<f64>, normal: Unit<Vector3<f64>>, texcoords: Vector2<f64>) -> Self {
        Self {
            position,
            normal,
            texcoords,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum VertexData {
    PNT([VertexPNT; 3]),
    Position([Point3<f64>; 3]),
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Triangle {
    #[serde(alias = "vertices")]
    vertex_data: VertexData,
    transform: Transform,
    pub material: Material,

    pub children: Option<Vec<Object3D>>,
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            vertex_data: VertexData::Position([
                Point3::origin(),
                Point3::origin(),
                Point3::origin(),
            ]),
            transform: Transform::default(),
            material: Material::default(),

            children: None,
        }
    }
}

impl Triangle {
    pub fn new(
        positions: [Point3<f64>; 3],
        normals: [Unit<Vector3<f64>>; 3],
        texcoords: [Vector2<f64>; 3],
        transform: Transform,
        material: Material,
    ) -> Self {
        let vertex_data = VertexData::PNT([
            VertexPNT::new(positions[0], normals[0], texcoords[0]),
            VertexPNT::new(positions[1], normals[1], texcoords[1]),
            VertexPNT::new(positions[2], normals[2], texcoords[2]),
        ]);

        Self {
            vertex_data,
            transform,
            material,

            children: None,
        }
    }

    pub fn new_with_positions(
        positions: [Point3<f64>; 3],
        transform: Transform,
        material: Material,
    ) -> Self {
        Self {
            vertex_data: VertexData::Position(positions),
            transform,
            material,

            children: None,
        }
    }

    pub fn compute_normal(positions: [Point3<f64>; 3]) -> Unit<Vector3<f64>> {
        let edge1 = positions[1] - positions[0];
        let edge2 = positions[2] - positions[0];

        Unit::new_normalize(edge1.cross(&edge2))
    }

    pub fn add_child(&mut self, object: Object3D) {
        if let Some(children) = self.children.as_mut() {
            children.push(object);
        }
    }

    pub fn flatten_to_world(self, transform: &Transform) -> Vec<Box<dyn RaytracingObject>> {
        let transform = transform * self.transform;

        let mut objects: Vec<Box<dyn RaytracingObject>> = Vec::new();

        if let Some(children) = self.children {
            for child in children {
                let child_objects: Vec<Box<dyn RaytracingObject>> =
                    child.flatten_to_world(&transform);
                objects.extend(child_objects);
            }
        }

        match self.vertex_data {
            VertexData::PNT(vertex_data) => {
                objects.push(Box::new(RaytracingTriangle::new(
                    vertex_data,
                    transform,
                    self.material,
                )));
            }
            VertexData::Position(vertices) => {
                objects.push(Box::new(RaytracingTriangle::new_with_positions(
                    vertices,
                    transform,
                    self.material,
                )));
            }
        }

        objects
    }
}

#[derive(Debug)]
pub struct RaytracingTriangle {
    vertex_data: [VertexPNT; 3],
    world_transform: Transform,
    material: Material,
}

impl RaytracingTriangle {
    fn new(vertex_data: [VertexPNT; 3], world_transform: Transform, material: Material) -> Self {
        Self {
            vertex_data,
            world_transform,
            material,
        }
    }

    fn new_with_positions(
        positions: [Point3<f64>; 3],
        world_transform: Transform,
        material: Material,
    ) -> Self {
        let normals = [Triangle::compute_normal(positions); 3];
        let texcoords = [Vector2::zero(); 3];

        let vertex_data = [
            VertexPNT::new(positions[0], normals[0], texcoords[0]),
            VertexPNT::new(positions[1], normals[1], texcoords[1]),
            VertexPNT::new(positions[2], normals[2], texcoords[2]),
        ];

        Self::new(vertex_data, world_transform, material)
    }
}

impl HasMaterial for RaytracingTriangle {
    fn get_material(&self) -> &Material {
        &self.material
    }
}

impl Transformed for RaytracingTriangle {
    fn get_transform(&self) -> &Transform {
        &self.world_transform
    }
}

impl Intersectable for RaytracingTriangle {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let edge1 = self.vertex_data[1].position - self.vertex_data[0].position;
        let edge2 = self.vertex_data[2].position - self.vertex_data[0].position;
        let p_vec = ray.direction.cross(&edge2);
        let det = edge1.dot(&p_vec);

        if match (self.material.side(), ray.ray_type) {
            (MaterialSide::Both, _) | (_, RayType::Shadow) => det.abs() < EPSILON,
            (MaterialSide::Front, _) => det < EPSILON,
            (MaterialSide::Back, _) => -det < EPSILON,
        } {
            return None;
        }

        let t_vec = ray.origin - self.vertex_data[0].position;
        let u = t_vec.dot(&p_vec) / det;
        if u < 0.0 || 1.0 < u {
            return None;
        }

        let q_vec = t_vec.cross(&edge1);
        let v = ray.direction.dot(&q_vec) / det;
        if v < 0.0 || 1.0 < u + v {
            return None;
        }

        let distance = edge2.dot(&q_vec) / det;

        Some(Intersection::new_with_data(
            self,
            distance,
            IntermediateData::Barycentric(u, v, 1.0 - u - v),
        ))
    }
}

impl Primitive for RaytracingTriangle {
    fn into_bounded_object(self: Box<Self>) -> ObjectWithBounds {
        let mut min = self.vertex_data[0].position;
        let mut max = min;
        for VertexPNT { position, .. } in self.vertex_data[1..].iter() {
            min.x = min.x.min(position.x);
            min.y = min.y.min(position.y);
            min.z = min.z.min(position.z);

            max.x = max.x.max(position.x);
            max.y = max.y.max(position.y);
            max.z = max.z.max(position.z);
        }
        let bounding_volume =
            BoundingVolume::from_bounds_and_transform(min, max, self.get_transform());

        ObjectWithBounds::bounded(self, bounding_volume)
    }

    fn surface_normal(
        &self,
        _object_hit_point: &Point3<f64>,
        intermediate: IntermediateData,
    ) -> Unit<Vector3<f64>> {
        let (u, v, w) = match intermediate {
            IntermediateData::Barycentric(u, v, w) => (u, v, w),
            _ => unreachable!(),
        };

        Unit::new_normalize(
            w * self.vertex_data[0].normal.into_inner()
                + u * self.vertex_data[1].normal.into_inner()
                + v * self.vertex_data[2].normal.into_inner(),
        )
    }

    fn uv(
        &self,
        _object_hit_point: &Point3<f64>,
        _object_normal: &Unit<Vector3<f64>>,
        intermediate: IntermediateData,
    ) -> Vector2<f64> {
        let (u, v, w) = match intermediate {
            IntermediateData::Barycentric(u, v, w) => (u, v, w),
            _ => unreachable!(),
        };

        w * self.vertex_data[0].texcoords
            + u * self.vertex_data[1].texcoords
            + v * self.vertex_data[2].texcoords
    }
}
