use nalgebra::{Point2, Point3, Unit, Vector2, Vector3};
use rand::Rng;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, TAU};
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
    debug_assert!(0.0 <= max_angle && max_angle <= FRAC_PI_2);

    if max_angle < EPSILON {
        return *direction;
    }

    let mut rng = rand::thread_rng();

    let theta = (rng.gen::<f64>()).acos();
    let theta = theta * max_angle / FRAC_PI_2;
    let z = theta.cos();
    let radius = theta.sin();

    let phi = rng.gen::<f64>() * TAU;

    let u = direction.cross(&Vector3::z_axis());
    let mag = u.magnitude();
    if mag < EPSILON {
        return Unit::new_normalize(Vector3::new(
            radius * phi.cos(),
            radius * phi.sin(),
            direction.z.signum() * z,
        ));
    }

    let w = direction.into_inner();
    let u = u.normalize();
    let v = direction.cross(&u).normalize();

    Unit::new_normalize(u * radius * phi.cos() + v * radius * phi.sin() + w * z)
}

#[cfg(test)]
mod test {
    use super::*;
    use more_asserts::assert_le;
    use std::f64::consts::PI;

    const PRECISION: f64 = 1e-6;

    #[test]
    fn it_samples_a_hemisphere() {
        for _ in 0..10_000 {
            let vec: Unit<Vector3<f64>> = Unit::new_normalize(Vector3::new_random());
            let sampled = cosine_sample_hemisphere(&vec);
            let dot = sampled.dot(&vec);

            assert_le!(dot.min(1.0).acos(), PI + PRECISION);
        }
    }

    #[test]
    fn it_samples_a_cone() {
        let mut rng = rand::thread_rng();

        for _ in 0..10_000 {
            let direction: Unit<Vector3<f64>> = Unit::new_normalize(Vector3::new_random());
            let max_angle = rng.gen::<f64>() * FRAC_PI_2;
            let sampled = uniform_sample_cone(&direction, max_angle);
            let dot = sampled.dot(&direction);

            assert_le!(dot.min(1.0).acos(), max_angle + PRECISION);
        }
    }

    #[test]
    fn it_samples_a_cone_z_direction() {
        let mut rng = rand::thread_rng();

        // +z, random max angle
        let direction = Vector3::z_axis();
        for _ in 0..10_000 {
            let max_angle = rng.gen::<f64>() * FRAC_PI_2;
            let sampled = uniform_sample_cone(&direction, max_angle);
            let dot = sampled.dot(&direction);

            assert_le!(dot.min(1.0).acos(), max_angle + PRECISION);
        }

        // -z, random max angle
        let direction = -Vector3::z_axis();
        for _ in 0..10_000 {
            let max_angle = rng.gen::<f64>() * FRAC_PI_2;
            let sampled = uniform_sample_cone(&direction, max_angle);
            let dot = sampled.dot(&direction);

            assert_le!(dot.min(1.0).acos(), max_angle + PRECISION);
        }
    }

    #[test]
    fn it_samples_a_cone_angle_edges() {
        // random direction, 0 max angle
        let zero_angle = 0.0;
        for _ in 0..10_000 {
            let direction: Unit<Vector3<f64>> = Unit::new_normalize(Vector3::new_random());
            let sampled = uniform_sample_cone(&direction, zero_angle);
            let dot = sampled.dot(&direction);

            assert_le!(dot.min(1.0).acos(), zero_angle + PRECISION);
        }

        // random direction, PI/2 max angle
        for _ in 0..10_000 {
            let direction: Unit<Vector3<f64>> = Unit::new_normalize(Vector3::new_random());
            let sampled = uniform_sample_cone(&direction, FRAC_PI_2);
            let dot = sampled.dot(&direction);

            assert_le!(dot.min(1.0).acos(), FRAC_PI_2 + PRECISION);
        }
    }

    #[test]
    fn it_samples_a_cone_angle_edges_z_direction() {
        let zero_angle = 0.0;

        let positive_z = Vector3::z_axis();
        let negative_z = -Vector3::z_axis();

        // +z, 0 max angle
        for _ in 0..10_000 {
            let sampled = uniform_sample_cone(&positive_z, zero_angle);
            let dot = sampled.dot(&positive_z);

            assert_le!(dot.min(1.0).acos(), zero_angle + PRECISION);
        }

        // -z, 0 max angle
        for _ in 0..10_000 {
            let sampled = uniform_sample_cone(&negative_z, zero_angle);
            let dot = sampled.dot(&negative_z);

            assert_le!(dot.min(1.0).acos(), zero_angle + PRECISION);
        }

        // +z, PI/2 max angle
        for _ in 0..10_000 {
            let sampled = uniform_sample_cone(&positive_z, FRAC_PI_2);
            let dot = sampled.dot(&positive_z);

            assert_le!(dot.min(1.0).acos(), FRAC_PI_2 + PRECISION);
        }

        // -z, PI/2 max angle
        for _ in 0..10_000 {
            let sampled = uniform_sample_cone(&negative_z, FRAC_PI_2);
            let dot = sampled.dot(&negative_z);

            assert_le!(dot.min(1.0).acos(), FRAC_PI_2 + PRECISION);
        }
    }
}
