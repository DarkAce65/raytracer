mod cube;
mod plane;
mod sphere;

use crate::raytrace::{Object3D, Ray};
use nalgebra::{Point3, Unit, Vector3, Vector4};
use std::marker::{Send, Sync};

pub use cube::*;
pub use plane::*;
pub use sphere::*;

fn quadratic(a: f64, b: f64, c: f64) -> Option<(f64, f64)> {
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

pub struct Intersection {
    pub distance: f64,
    pub object: Box<dyn Primitive>,
}

pub trait Drawable {
    fn color(&self) -> Vector4<f64>;
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn surface_normal(&self, hit_point: &Point3<f64>) -> Unit<Vector3<f64>>;
}

pub trait Primitive: Send + Sync + Object3D + Drawable + Intersectable {}
impl<T> Primitive for T where T: Send + Sync + Object3D + Drawable + Intersectable {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_solves_quadratic_eqs() {
        assert_eq!(quadratic(1.0, 2.0, 1.0), Some((-1.0, -1.0)));
        assert_eq!(quadratic(1.0, -6.0, 9.0), Some((3.0, 3.0)));
        assert_eq!(quadratic(4.0, 4.0, 1.0), Some((-0.5, -0.5)));
        assert_eq!(quadratic(2.0, -25.0, 12.0), Some((0.5, 12.0)));
        assert_eq!(quadratic(1.0, 1.0, 1.0), None);
    }
}
