use crate::{transform::Transform, Vector3};

use super::aabb::AxisAlignedBoundingBox;

pub enum ColliderShape {
    AABB(AxisAlignedBoundingBox),
}

impl ColliderShape {
    pub fn get_correction_vec(
        a: &Self,
        a_transform: &Transform,
        b: &Self,
        b_transform: &Transform,
    ) -> Option<Vector3> {
        match (a, b) {
            (ColliderShape::AABB(a), ColliderShape::AABB(b)) => {
                AxisAlignedBoundingBox::aabb_correction_vec(a, a_transform, b, b_transform)
            }
        }
    }

    pub fn contains(
        &self,
        transform: &Transform,
        other: &ColliderShape,
        other_transform: &Transform,
    ) -> bool {
        match (self, other) {
            (ColliderShape::AABB(aabb), ColliderShape::AABB(other_aabb)) => {
                AxisAlignedBoundingBox::contains_aabb(aabb, transform, other_aabb, other_transform)
            }
        }
    }
}
