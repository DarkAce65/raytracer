mod bounds;
mod material;
mod texture;
mod transform;

pub use bounds::{BoundedObject, BoundingVolume, KdTreeAccelerator, ObjectWithBounds};
pub use material::{Material, MaterialSide, PhongMaterial, PhysicalMaterial};
pub use texture::Texture;
pub use transform::{Transform, Transformed};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn iter(initial_axis: Axis) -> impl Iterator<Item = usize> {
        let start: usize = initial_axis.into();

        (start..(start + 3)).map(|a| a % 3)
    }
}

impl From<Axis> for usize {
    fn from(axis: Axis) -> Self {
        match axis {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
}

impl From<usize> for Axis {
    fn from(axis: usize) -> Self {
        match axis {
            0 => Axis::X,
            1 => Axis::Y,
            2 => Axis::Z,
            _ => panic!("{:?} is not a valid 3D axis", axis),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AxisDirection(pub Axis, pub bool);
