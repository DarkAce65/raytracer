use nalgebra::Vector3;
use std::f64::consts::PI;

// Trowbridge-Reitz GGX normal distribution function
pub fn ndf(n_dot_h: f64, roughness: f64) -> f64 {
    let a = roughness * roughness;
    let a2 = a * a;

    let n_dot_h2 = n_dot_h * n_dot_h;
    let denom = n_dot_h2 * (a2 - 1.0) + 1.0;
    let denom = PI * denom * denom;

    a2 / denom
}

// Smith's Schlick-GGX geometry function
pub fn geometry_function(n_dot_v: f64, n_dot_l: f64, roughness: f64) -> f64 {
    let r = roughness + 1.0;
    let k = r * r / 8.0;

    let ggx1 = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let ggx2 = n_dot_l / (n_dot_l * (1.0 - k) + k);

    ggx1 * ggx2
}

// Fresnel-Schlick equation
pub fn fresnel(n_dot_v: f64, base_reflectivity: Vector3<f64>) -> Vector3<f64> {
    base_reflectivity + (Vector3::repeat(1.0) - base_reflectivity) * (1.0 - n_dot_v).powf(5.0)
}
