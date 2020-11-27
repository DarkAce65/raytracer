mod blur;
mod physical_material_equations;
mod rays;
mod sampling;

use nalgebra::Vector3;
use num_traits::Float;

pub use physical_material_equations::{fresnel, geometry_function, ndf};
pub use rays::{reflect, refract};
pub use sampling::{cosine_sample_hemisphere, uniform_sample_cone};

const ALPHA_BIT_MASK: u32 = 255 << 24;
const BOX_BLUR_ITERATIONS: usize = 3;

pub fn to_argb_u32(rgb: Vector3<f64>) -> u32 {
    let r = (rgb.x * 255.0) as u32;
    let g = (rgb.y * 255.0) as u32;
    let b = (rgb.z * 255.0) as u32;
    ALPHA_BIT_MASK | r << 16 | g << 8 | b
}

pub fn gamma_correct(color: Vector3<f64>, gamma: f64) -> Vector3<f64> {
    color.map(|c| c.powf(1.0 / gamma))
}

pub fn lerp<F: Float>(x0: F, x1: F, t: F) -> F {
    x0 - x0 * t + x1 * t
}

pub fn remap_value<F: Float>(num: F, domain: (F, F), range: (F, F)) -> F {
    assert!(domain.0 < domain.1, "domain values must be of the form (min, max) - range values can be swapped for this behavior");

    (num - domain.0) * (range.1 - range.0) / (domain.1 - domain.0) + range.0
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

#[cfg(test)]
mod test {
    use super::*;

    #[allow(clippy::shadow_unrelated)]
    #[test]
    fn it_converts_color_vecs_to_u32() {
        let color = ALPHA_BIT_MASK;
        assert_eq!(to_argb_u32(Vector3::from([0.0, 0.0, 0.0])), color);
        let color = ALPHA_BIT_MASK | 255 << 16 | 255 << 8 | 255;
        assert_eq!(to_argb_u32(Vector3::from([1.0, 1.0, 1.0])), color);
        let color = ALPHA_BIT_MASK | 255;
        assert_eq!(to_argb_u32(Vector3::from([0.0, 0.0, 1.0])), color);
        let color = ALPHA_BIT_MASK | 255 << 16 | 255;
        assert_eq!(to_argb_u32(Vector3::from([1.0, 0.0, 1.0])), color);
    }

    #[test]
    fn it_maps_numbers() {
        assert_eq!(remap_value(1.0, (0.0, 1.0), (0.0, 5.0)), 5.0);
        assert_eq!(remap_value(0.5, (0.0, 1.0), (0.0, 5.0)), 2.5);
        assert_eq!(remap_value(0.5, (0.0, 1.0), (0.0, 10.0)), 5.0);
        assert_eq!(remap_value(0.5, (0.0, 0.5), (0.0, 10.0)), 10.0);
        assert_eq!(remap_value(-1.0, (0.0, 1.0), (0.0, 10.0)), -10.0);
        assert_eq!(remap_value(2.0, (0.0, 1.0), (0.0, 10.0)), 20.0);
    }

    #[test]
    fn it_solves_quadratic_eqs() {
        assert_eq!(quadratic(1.0, 2.0, 1.0), Some((-1.0, -1.0)));
        assert_eq!(quadratic(1.0, -6.0, 9.0), Some((3.0, 3.0)));
        assert_eq!(quadratic(4.0, 4.0, 1.0), Some((-0.5, -0.5)));
        assert_eq!(quadratic(2.0, -25.0, 12.0), Some((0.5, 12.0)));
        assert_eq!(quadratic(1.0, 1.0, 1.0), None);
    }
}
