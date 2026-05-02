use crate::{collision::aabb::AxisAlignedBoundingBox, transform::Transform, Component, Vector3};

use super::shape::ColliderShape;

pub struct Collider {
    shape: ColliderShape,
    dynamic: bool, // false -> collider is static
}

impl Collider {
    pub fn new(shape: ColliderShape, dynamic: bool) -> Self {
        Self { shape, dynamic }
    }

    pub fn contains(
        &self,
        transform: &Transform,
        other: &Self,
        other_transform: &Transform,
    ) -> bool {
        ColliderShape::contains(&self.shape, transform, &other.shape, other_transform)
    }

    pub fn get_correction_vec(
        a: &Self,
        a_transform: &Transform,
        b: &Self,
        b_transform: &Transform,
    ) -> Option<Vector3> {
        ColliderShape::get_correction_vec(&a.shape, a_transform, &b.shape, b_transform)
    }

    pub fn dynamic(&self) -> bool {
        self.dynamic
    }

    /// Construct a new AABB shaped collider: (axis-aligned bounding box)
    pub fn new_aabb(position: Vector3, dimensions: Vector3, dynamic: bool) -> Self {
        let min = position - dimensions;
        let max = position + dimensions;
        Self {
            shape: ColliderShape::AABB(AxisAlignedBoundingBox::new(min, max)),
            dynamic,
        }
    }
}

impl Component for Collider {
    fn on_start(&mut self, scene: &mut crate::Scene, context: crate::OnStartContext) {}

    fn on_update(&mut self, scene: &mut crate::Scene, context: crate::OnUpdateContext) {}

    fn on_event(&mut self, scene: &mut crate::Scene, context: crate::OnEventContext) {}
}
