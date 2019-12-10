mod bounds;
mod material;
mod transform;

use nalgebra::{Point2, Point3, Unit, Vector2, Vector3};
use rand::Rng;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use std::f64::EPSILON;

pub use bounds::*;
pub use material::*;
pub use transform::*;

pub fn reflect(incident: &Vector3<f64>, normal: &Vector3<f64>) -> Unit<Vector3<f64>> {
    Unit::new_normalize(incident - 2.0 * incident.dot(&normal) * normal)
}

pub fn refract(
    incident: &Vector3<f64>,
    normal: &Vector3<f64>,
    eta: f64,
) -> Option<Unit<Vector3<f64>>> {
    let n_dot_i = normal.dot(&incident);
    let refraction_normal = if n_dot_i < 0.0 { *normal } else { -*normal };
    let eta = if n_dot_i < 0.0 { eta } else { 1.0 / eta };
    let n_dot_i = n_dot_i.abs();

    let k = 1.0 - eta * eta * (1.0 - n_dot_i * n_dot_i);

    if k < 0.0 {
        None
    } else {
        Some(Unit::new_normalize(
            incident * eta - refraction_normal * (eta * n_dot_i - k.sqrt()),
        ))
    }
}

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

fn concentric_sample_disk() -> Point2<f64> {
    let rnd: Vector2<f64> = 2.0 * Vector2::new_random() - Vector2::from([1.0, 1.0]);

    if rnd.x == 0.0 && rnd.y == 0.0 {
        return Point2::origin();
    }

    let (r, theta) = if rnd.x.abs() > rnd.y.abs() {
        (rnd.x, FRAC_PI_2 * (rnd.y / rnd.x))
    } else {
        (rnd.y, FRAC_PI_2 - FRAC_PI_4 * (rnd.x / rnd.y))
    };

    r * Point2::from([theta.cos(), theta.sin()])
}

// Sample a hemisphere with a cosine weight in the direction of the given direction using Malley's method
#[allow(dead_code)]
pub fn cosine_sample_hemisphere(direction: &Unit<Vector3<f64>>) -> Unit<Vector3<f64>> {
    let p = concentric_sample_disk();
    let p = Point3::from([p.x, p.y, (1.0 - p.x * p.x - p.y * p.y).sqrt()]);

    let w = direction.into_inner();
    let u = if w.x.abs() > EPSILON {
        direction.cross(&Vector3::y_axis())
    } else {
        direction.cross(&Vector3::x_axis())
    };
    let v = direction.cross(&u);

    Unit::new_normalize(u * p.x + v * p.y + w * p.z)
}

// Sample a cone in the direction of the given direction
pub fn uniform_sample_cone(direction: &Unit<Vector3<f64>>, max_angle: f64) -> Unit<Vector3<f64>> {
    let mut rng = rand::thread_rng();
    let rnd: f64 = rng.gen();
    let z = 1.0 - rnd + rnd * max_angle;
    let radius = (1.0 - z * z).sqrt();

    let phi = rng.gen::<f64>() * 2.0 * PI;

    let w = direction.into_inner();
    let u = if w.x.abs() > EPSILON {
        direction.cross(&Vector3::y_axis())
    } else {
        direction.cross(&Vector3::x_axis())
    };
    let v = direction.cross(&u);

    Unit::new_normalize(u * radius * phi.cos() + v * radius * phi.sin() + w * z)
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

            assert_le!(0.0, dot);
            assert_le!(dot, 1.0);

            i += 1;
        }
    }
}
