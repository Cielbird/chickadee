use crate::{transform::Transform, Vector3};

use super::r#box::SimpleBoxCollider;

pub enum ColliderShape {
    Box(SimpleBoxCollider),
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
                SimpleBoxCollider::box_correction_vec(a, a_transform, b, b_transform)
            }
        }
    }
}
