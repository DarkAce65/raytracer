use super::{HasMaterial, Loadable, Object3D, Primitive};
use crate::core::{Bounds, Material, Texture, Transform, Transformed};
use crate::ray_intersection::{IntermediateData, Intersectable, Intersection, Ray};
use nalgebra::{Point3, Unit, Vector2, Vector3};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    #[serde(default)]
    transform: Transform,
    children: Vec<Box<dyn Object3D>>,
}

impl HasMaterial for Group {
    fn get_material(&self) -> &Material {
        unimplemented!()
    }

    fn get_material_mut(&mut self) -> &mut Material {
        unimplemented!()
    }
}

impl Loadable for Group {
    fn load_assets(
        &mut self,
        _asset_base: &Path,
        _textures: &mut HashMap<String, Texture>,
    ) -> bool {
        false
    }
}

impl Transformed for Group {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }
}

impl Intersectable for Group {
    fn intersect(&self, _ray: &Ray) -> Option<Intersection> {
        None
    }
}

impl Primitive for Group {
    fn make_bounding_volume(&self, _transform: &Transform) -> Bounds {
        Bounds::NonIntersectable
    }

    fn get_children(&self) -> Option<&Vec<Box<dyn Object3D>>> {
        Some(self.children.as_ref())
    }

    fn get_children_mut(&mut self) -> Option<&mut Vec<Box<dyn Object3D>>> {
        Some(self.children.as_mut())
    }

    fn surface_normal(
        &self,
        _object_hit_point: &Point3<f64>,
        _intermediate: IntermediateData,
    ) -> Unit<Vector3<f64>> {
        unimplemented!()
    }

    fn uv(
        &self,
        _object_hit_point: &Point3<f64>,
        _object_normal: &Unit<Vector3<f64>>,
        _intermediate: IntermediateData,
    ) -> Vector2<f64> {
        unimplemented!()
    }
}
