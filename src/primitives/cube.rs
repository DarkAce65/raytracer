use super::{HasMaterial, Object3D, Primitive, RaytracingObject};
use crate::core::{
    Axis, AxisDirection, BoundingVolume, Material, MaterialSide, ObjectWithBounds, Transform,
    Transformed,
};
use crate::ray_intersection::{IntermediateData, Intersectable, Intersection, Ray, RayType};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Cube {
    size: f64,
    transform: Transform,
    pub material: Material,

    pub children: Option<Vec<Object3D>>,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            size: 1.0,
            transform: Transform::default(),
            material: Material::default(),

            children: None,
        }
    }
}

impl Cube {
    pub fn new(size: f64, transform: Transform, material: Material) -> Self {
        Self {
            size,
            transform,
            material,
            ..Cube::default()
        }
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

        objects.push(Box::new(RaytracingCube::new(
            self.size,
            transform,
            self.material,
        )));

        objects
    }
}

#[derive(Debug)]
pub struct RaytracingCube {
    size: f64,
    world_transform: Transform,
    material: Material,
}

impl RaytracingCube {
    pub fn new(size: f64, world_transform: Transform, material: Material) -> Self {
        Self {
            size,
            world_transform,
            material,
        }
    }
}

impl HasMaterial for RaytracingCube {
    fn get_material(&self) -> &Material {
        &self.material
    }
}

impl Transformed for RaytracingCube {
    fn get_transform(&self) -> &Transform {
        &self.world_transform
    }
}

impl Intersectable for RaytracingCube {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let ray_sign = ray.direction.map(|c| c.signum());
        let half = self.size / 2.0;

        let mut hit_axis_near = AxisDirection(Axis::X, ray_sign.x < 0.0);
        let mut hit_axis_far = AxisDirection(Axis::X, ray_sign.x > 0.0);

        let d_near = (-ray.origin.x - ray_sign.x * half) / ray.direction.x;
        let d_far = (-ray.origin.x + ray_sign.x * half) / ray.direction.x;

        let dy_near = (-ray.origin.y - ray_sign.y * half) / ray.direction.y;
        let dy_far = (-ray.origin.y + ray_sign.y * half) / ray.direction.y;

        if dy_far < d_near || d_far < dy_near {
            return None;
        }

        let d_near = if dy_near > d_near {
            hit_axis_near = AxisDirection(Axis::Y, ray_sign.y < 0.0);
            dy_near
        } else {
            d_near
        };
        let d_far = if d_far > dy_far {
            hit_axis_far = AxisDirection(Axis::Y, ray_sign.y > 0.0);
            dy_far
        } else {
            d_far
        };

        let dz_near = (-ray.origin.z - ray_sign.z * half) / ray.direction.z;
        let dz_far = (-ray.origin.z + ray_sign.z * half) / ray.direction.z;

        if dz_far < d_near || d_far < dz_near {
            return None;
        }

        let d_near = if dz_near > d_near {
            hit_axis_near = AxisDirection(Axis::Z, ray_sign.z < 0.0);
            dz_near
        } else {
            d_near
        };
        let d_far = if d_far > dz_far {
            hit_axis_far = AxisDirection(Axis::Z, ray_sign.z > 0.0);
            dz_far
        } else {
            d_far
        };

        debug_assert!(d_near <= d_far);

        let (d, hit_axis) = match (self.material.side(), ray.ray_type) {
            (MaterialSide::Both, _) | (_, RayType::Shadow) => {
                if d_near < 0.0 {
                    (d_far, hit_axis_far)
                } else {
                    (d_near, hit_axis_near)
                }
            }
            (MaterialSide::Front, _) => (d_near, hit_axis_near),
            (MaterialSide::Back, _) => (d_far, hit_axis_far),
        };
        if d < 0.0 {
            return None;
        }

        Some(Intersection::new_with_data(
            self,
            d,
            IntermediateData::CubeHitFace(hit_axis),
        ))
    }
}

impl Primitive for RaytracingCube {
    fn into_bounded_object(self: Box<Self>) -> ObjectWithBounds {
        let half = self.size / 2.0;
        let bounding_volume = BoundingVolume::from_bounds_and_transform(
            Point3::from([-half; 3]),
            Point3::from([half; 3]),
            self.get_transform(),
        );

        ObjectWithBounds::bounded(self, bounding_volume)
    }

    fn surface_normal(
        &self,
        _object_hit_point: &Point3<f64>,
        intermediate: IntermediateData,
    ) -> Unit<Vector3<f64>> {
        match intermediate {
            IntermediateData::CubeHitFace(axis_direction) => {
                let AxisDirection(axis, positive) = axis_direction;
                let normal = match axis {
                    Axis::X => Vector3::x_axis(),
                    Axis::Y => Vector3::y_axis(),
                    Axis::Z => Vector3::z_axis(),
                };

                if positive {
                    normal
                } else {
                    -normal
                }
            }
            _ => unreachable!(),
        }
    }

    fn uv(
        &self,
        object_hit_point: &Point3<f64>,
        _object_normal: &Unit<Vector3<f64>>,
        intermediate: IntermediateData,
    ) -> Vector2<f64> {
        let hit_point = object_hit_point.coords.map(|c| c / self.size + 0.5);

        match intermediate {
            IntermediateData::CubeHitFace(axis_direction) => {
                let AxisDirection(axis, positive) = axis_direction;

                if positive {
                    match axis {
                        Axis::X => Vector2::new(-hit_point.z, hit_point.y),
                        Axis::Y => Vector2::new(hit_point.x, -hit_point.z),
                        Axis::Z => Vector2::new(hit_point.x, hit_point.y),
                    }
                } else {
                    match axis {
                        Axis::X => Vector2::new(hit_point.z, hit_point.y),
                        Axis::Y => Vector2::new(hit_point.x, hit_point.z),
                        Axis::Z => Vector2::new(-hit_point.x, hit_point.y),
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
