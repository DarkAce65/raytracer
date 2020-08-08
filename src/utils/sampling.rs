use nalgebra::{Point2, Point3, Unit, Vector2, Vector3};
use rand::Rng;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use std::f64::EPSILON;

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
