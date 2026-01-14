use crate::{transform::Transform, Vector3};

use super::r#box::BoxCollider;

pub enum ColliderShape {
    Box(BoxCollider),
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
                BoxCollider::box_correction_vec(a, a_transform, b, b_transform)
            }
        }
    }
}
