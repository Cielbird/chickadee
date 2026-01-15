use crate::{collision::r#box::AxisAlignedBox, transform::Transform, Component, Vector3};

use super::shape::ColliderShape;

pub struct Collider {
    shape: ColliderShape,
    dynamic: bool, // false -> collider is static
}

impl Collider {
    pub fn new(shape: ColliderShape, dynamic: bool) -> Self {
        Self { shape, dynamic }
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
        Self {
            shape: ColliderShape::Box(AxisAlignedBox::new(position, dimensions)),
            dynamic,
        }
    }
}

// Colliders get special treatment from the Scene
impl Component for Collider {
    fn on_start(&mut self, _scene: &mut crate::Scene, _context: crate::OnStartContext) {}

    fn on_update(&mut self, _scene: &mut crate::Scene, _context: crate::OnUpdateContext) {}

    fn on_event(&mut self, _scene: &mut crate::Scene, _context: crate::OnEventContext) {}
}
