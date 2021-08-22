use nalgebra::{Unit, Vector3};

pub fn reflect(incident: &Vector3<f64>, normal: &Vector3<f64>) -> Unit<Vector3<f64>> {
    Unit::new_normalize(incident - 2.0 * incident.dot(normal) * normal)
}

pub fn refract(
    incident: &Vector3<f64>,
    normal: &Vector3<f64>,
    eta: f64,
) -> Option<Unit<Vector3<f64>>> {
    let n_dot_i = normal.dot(incident);
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
