use crate::primitives::Primitive;
use derive_builder::Builder;
use nalgebra::{Affine3, Matrix4, Point3, Rotation3, Translation3, Unit, Vector3};
use rand::Rng;
use std::default::Default;
use std::f64::consts::PI;
use std::result::Result;

pub const EPSILON: f64 = 1e-10;

pub fn quadratic(a: f64, b: f64, c: f64) -> Option<(f64, f64)> {
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        None
    } else if discriminant == 0.0 {
        Some((-0.5 * b / a, -0.5 * b / a))
    } else {
        let q = -0.5 * (b + b.signum() * discriminant.sqrt());
        let r0 = q / a;
        let r1 = c / q;
        Some((r0.min(r1), r0.max(r1)))
    }
}

pub fn cosine_sample_hemisphere(normal: &Unit<Vector3<f64>>) -> Unit<Vector3<f64>> {
    let mut rng = rand::thread_rng();

    let theta = 2.0 * PI * rng.gen::<f64>();
    let r = rng.gen::<f64>();
    let rs = r.sqrt();

    let w = normal.into_inner();
    let u = if w.x.abs() > EPSILON {
        normal.cross(&Vector3::y_axis())
    } else {
        normal.cross(&Vector3::x_axis())
    };

    let v = normal.cross(&u);
    Unit::new_normalize(u * rs * theta.cos() + v * rs * theta.sin() + (1.0 - r).sqrt() * w)
}

#[derive(Builder, Copy, Clone, Debug)]
#[builder(default, build_fn(skip))]
pub struct Transform {
    pub position: Point3<f64>,
    rotation: (f64, Unit<Vector3<f64>>),
    scale: Vector3<f64>,

    #[builder(setter(skip))]
    matrix: Affine3<f64>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            rotation: (0.0, Vector3::y_axis()),
            scale: Vector3::repeat(1.0),
            matrix: Affine3::identity(),
        }
    }
}

impl TransformBuilder {
    pub fn build(&self) -> Result<Transform, String> {
        let __default: Transform = Default::default();
        let position = match self.position {
            Some(ref value) => Clone::clone(value),
            None => __default.position,
        };
        let rotation = match self.rotation {
            Some(ref value) => Clone::clone(value),
            None => __default.rotation,
        };
        let scale = match self.scale {
            Some(ref value) => Clone::clone(value),
            None => __default.scale,
        };
        let matrix: Affine3<f64> = Translation3::from(position.coords)
            * Rotation3::from_axis_angle(&rotation.1, rotation.0.to_radians())
            * Affine3::from_matrix_unchecked(Matrix4::new_nonuniform_scaling(&scale));

        Ok(Transform {
            position,
            rotation,
            scale,
            matrix,
        })
    }
}

pub trait Object3D {
    fn transform(&self) -> Transform;
}

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f64>,
    pub direction: Vector3<f64>,
}

impl Ray {
    pub fn transform(&self, transform: Affine3<f64>) -> Ray {
        let origin = transform * self.origin;
        let direction = transform * self.direction;

        Ray { origin, direction }
    }
}

#[derive(Debug)]
pub struct Intersection {
    pub distance: f64,
    pub object: Box<dyn Primitive>,
}

#[cfg(test)]
mod test {
    use super::*;
    use more_asserts::assert_le;

    #[test]
    fn it_solves_quadratic_eqs() {
        assert_eq!(quadratic(1.0, 2.0, 1.0), Some((-1.0, -1.0)));
        assert_eq!(quadratic(1.0, -6.0, 9.0), Some((3.0, 3.0)));
        assert_eq!(quadratic(4.0, 4.0, 1.0), Some((-0.5, -0.5)));
        assert_eq!(quadratic(2.0, -25.0, 12.0), Some((0.5, 12.0)));
        assert_eq!(quadratic(1.0, 1.0, 1.0), None);
    }

    #[test]
    fn it_generates_hemisphere_samples() {
        let mut i = 0;
        loop {
            if i >= 1000 {
                break;
            }

            let vec: Unit<Vector3<f64>> = Unit::new_normalize(Vector3::new_random());
            let sampled = cosine_sample_hemisphere(&vec);
            let dot = sampled.dot(&vec);

            assert_le!(0.0, dot,);
            assert_le!(dot, 1.0);

            i += 1;
        }
    }
}
