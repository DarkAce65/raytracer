use nalgebra::{Affine3, Matrix4, Rotation3, Translation3, Unit, Vector3};
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::default::Default;
use std::fmt;

#[derive(Copy, Clone, Debug, Serialize)]
pub struct Transform {
    matrix: Affine3<f64>,
    inv_matrix: Affine3<f64>,
}

impl Default for Transform {
    fn default() -> Self {
        let matrix = Affine3::identity();
        Self {
            matrix,
            inv_matrix: matrix.inverse(),
        }
    }
}

impl Transform {
    pub fn matrix(&self) -> Affine3<f64> {
        self.matrix
    }

    pub fn inverse(&self) -> Affine3<f64> {
        self.inv_matrix
    }

    pub fn inverse_transpose(&self) -> Affine3<f64> {
        Affine3::from_matrix_unchecked(
            nalgebra::convert::<Affine3<f64>, Matrix4<f64>>(self.inverse()).transpose(),
        )
    }

    fn set_matrix(&mut self, m: Affine3<f64>) -> &mut Self {
        self.matrix = m;
        self.inv_matrix = self.matrix.inverse();
        self
    }

    pub fn translate(&mut self, translation: Vector3<f64>) -> &mut Self {
        self.set_matrix(Translation3::from(translation) * self.matrix)
    }

    pub fn rotate(&mut self, angle: f64, axis: Unit<Vector3<f64>>) -> &mut Self {
        self.set_matrix(Rotation3::from_axis_angle(&axis, angle.to_radians()) * self.matrix)
    }

    pub fn scale(&mut self, scale: Vector3<f64>) -> &mut Self {
        self.set_matrix(
            Affine3::from_matrix_unchecked(Matrix4::new_nonuniform_scaling(&scale)) * self.matrix,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "lowercase"))]
enum SubTransform {
    Translate(Vector3<f64>),
    Rotate(f64, Unit<Vector3<f64>>),
    Scale(Vector3<f64>),
}

struct TransformVisitor;

impl<'de> Visitor<'de> for TransformVisitor {
    type Value = Transform;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Transform")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Transform, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut transform = Transform::default();
        loop {
            let next: Option<SubTransform> = seq.next_element()?;
            if let Some(next) = next {
                match next {
                    SubTransform::Translate(translation) => {
                        transform = *transform.translate(translation)
                    }
                    SubTransform::Rotate(angle, axis) => transform = *transform.rotate(angle, axis),
                    SubTransform::Scale(scale) => transform = *transform.scale(scale),
                }
            } else {
                break;
            }
        }

        Ok(transform)
    }
}

impl<'de> Deserialize<'de> for Transform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TransformVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nalgebra::{Affine3, Matrix4, Vector3};

    #[test]
    fn it_constructs_matrices() {
        let default = Transform::default();
        let translation = *Transform::default().translate(Vector3::from([1.0, 2.0, 3.0]));
        let rotation = *Transform::default().rotate(50.0, Vector3::y_axis());
        let scale = *Transform::default().scale(Vector3::from([1.0, 2.0, 3.0]));

        // Base transform matrix
        assert_eq!(default.matrix(), Affine3::identity());

        assert_eq!(
            translation.matrix(),
            Affine3::from_matrix_unchecked(Matrix4::new_translation(&Vector3::from([
                1.0, 2.0, 3.0
            ])))
        );
        assert_eq!(
            rotation.matrix(),
            Affine3::from_matrix_unchecked(Matrix4::from_axis_angle(
                &Vector3::y_axis(),
                50.0f64.to_radians()
            ))
        );
        assert_eq!(
            scale.matrix(),
            Affine3::from_matrix_unchecked(Matrix4::new_nonuniform_scaling(&Vector3::from([
                1.0, 2.0, 3.0
            ])))
        );

        // Inverse transform matrix
        assert_eq!(default.inverse(), Affine3::identity().inverse());

        assert_eq!(
            translation.inverse(),
            Affine3::from_matrix_unchecked(Matrix4::new_translation(&Vector3::from([
                1.0, 2.0, 3.0
            ])))
            .inverse()
        );
        assert_eq!(
            rotation.inverse(),
            Affine3::from_matrix_unchecked(Matrix4::from_axis_angle(
                &Vector3::y_axis(),
                50.0f64.to_radians()
            ))
            .inverse()
        );
        assert_eq!(
            scale.inverse(),
            Affine3::from_matrix_unchecked(Matrix4::new_nonuniform_scaling(&Vector3::from([
                1.0, 2.0, 3.0
            ])))
            .inverse()
        );

        // Inverse transpose transform matrix
        assert_eq!(
            default.inverse_transpose(),
            Affine3::from_matrix_unchecked(Matrix4::identity().transpose()).inverse()
        );

        assert_eq!(
            translation.inverse_transpose(),
            Affine3::from_matrix_unchecked(
                Matrix4::new_translation(&Vector3::from([1.0, 2.0, 3.0])).transpose()
            )
            .inverse()
        );
        assert_eq!(
            rotation.inverse_transpose(),
            Affine3::from_matrix_unchecked(
                Matrix4::from_axis_angle(&Vector3::y_axis(), 50.0f64.to_radians()).transpose()
            )
            .inverse()
        );
        assert_eq!(
            scale.inverse_transpose(),
            Affine3::from_matrix_unchecked(
                Matrix4::new_nonuniform_scaling(&Vector3::from([1.0, 2.0, 3.0])).transpose()
            )
            .inverse()
        );
    }

    #[test]
    fn it_constructs_complex_matrices() {
        let full = *Transform::default()
            .rotate(50.0, Vector3::y_axis())
            .scale(Vector3::from([3.0, 2.0, 1.0]))
            .translate(Vector3::from([5.0, 2.0, 3.0]));
        let translation_identity = *Transform::default()
            .translate(Vector3::from([1.0, 2.0, 3.0]))
            .translate(Vector3::from([-1.0, -2.0, -3.0]));
        let full_identity = *Transform::default()
            .rotate(50.0, Vector3::y_axis())
            .scale(Vector3::from([1.0, 2.0, 4.0]))
            .translate(Vector3::from([1.0, 2.0, 3.0]))
            .translate(Vector3::from([-1.0, -2.0, -3.0]))
            .scale(Vector3::from([1.0, 0.5, 0.25]))
            .rotate(-50.0, Vector3::y_axis());

        // Base transform matrix
        assert_eq!(
            full.matrix(),
            Affine3::from_matrix_unchecked(
                Matrix4::from_axis_angle(&Vector3::y_axis(), 50.0f64.to_radians())
                    .append_nonuniform_scaling(&Vector3::from([3.0, 2.0, 1.0]))
                    .append_translation(&Vector3::from([5.0, 2.0, 3.0]))
            )
        );
        assert_eq!(translation_identity.matrix(), Affine3::identity());
        assert_eq!(full_identity.matrix(), Affine3::identity());

        // Inverse transform matrix
        assert_eq!(
            full.inverse(),
            Affine3::from_matrix_unchecked(
                Matrix4::from_axis_angle(&Vector3::y_axis(), 50.0f64.to_radians())
                    .append_nonuniform_scaling(&Vector3::from([3.0, 2.0, 1.0]))
                    .append_translation(&Vector3::from([5.0, 2.0, 3.0]))
            )
            .inverse()
        );
        assert_eq!(
            translation_identity.inverse(),
            Affine3::identity().inverse()
        );
        assert_eq!(full_identity.inverse(), Affine3::identity().inverse());

        // Inverse transpose transform matrix
        assert_eq!(
            full.inverse_transpose(),
            Affine3::from_matrix_unchecked(
                Matrix4::from_axis_angle(&Vector3::y_axis(), 50.0f64.to_radians())
                    .append_nonuniform_scaling(&Vector3::from([3.0, 2.0, 1.0]))
                    .append_translation(&Vector3::from([5.0, 2.0, 3.0]))
                    .transpose()
            )
            .inverse()
        );
        assert_eq!(
            translation_identity.inverse_transpose(),
            Affine3::from_matrix_unchecked(Matrix4::identity().transpose()).inverse()
        );
        assert_eq!(
            full_identity.inverse_transpose(),
            Affine3::from_matrix_unchecked(Matrix4::identity().transpose()).inverse()
        );
    }
}
