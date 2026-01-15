use crate::{transform::Transform, Vector3};

use super::r#box::AxisAlignedBox;

pub enum ColliderShape {
    Box(AxisAlignedBox),
}

impl ColliderShape {
    pub fn get_correction_vec(
        a: &Self,
        a_transform: &Transform,
        b: &Self,
        b_transform: &Transform,
    ) -> Option<Vector3> {
        match (a, b) {
            (ColliderShape::Box(a), ColliderShape::Box(b)) => {
                AxisAlignedBox::box_correction_vec(a, a_transform, b, b_transform)
            }
        }
    }
}
