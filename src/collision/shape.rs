use crate::Vector3;

use super::r#box::BoxCollider;

pub enum ColliderShape {
    Box(BoxCollider),
}

impl ColliderShape {
    pub fn get_correction_vec(&self, other: &Self) -> Option<Vector3> {
        match self {
            ColliderShape::Box(box_collider) => box_collider.get_correction_vec(other),
        }
    }
}
