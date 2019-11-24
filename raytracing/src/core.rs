use crate::primitives::Primitive;
use nalgebra::{Point3, Unit, Vector3};
use rand::Rng;

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

    let r1 = 2.0 * std::f64::consts::PI * rng.gen::<f64>();
    let r2 = rng.gen::<f64>();
    let r2s = r2.sqrt();

    let w = normal.into_inner();
    let u = if w.x.abs() > EPSILON {
        normal.cross(&Vector3::y_axis())
    } else {
        normal.cross(&Vector3::x_axis())
    };

    let v = normal.cross(&u);
    Unit::new_normalize(u * r1.cos() * r2s + v * r1.sin() * r2s + w * (1.0 - r2).sqrt())
}

pub trait Object3D {
    fn position(&self) -> Point3<f64>;

    fn scale(&self) -> Vector3<f64> {
        unimplemented!()
    }

    fn rotation(&self) -> Vector3<f64> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f64>,
    pub direction: Unit<Vector3<f64>>,
}

#[derive(Debug)]
pub struct Intersection {
    pub distance: f64,
    pub object: Box<dyn Primitive>,
}

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
