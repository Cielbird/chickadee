use crate::{collision::r#box::BoxCollider, Component, Vector3};

use super::shape::ColliderShape;

pub struct Collider {
    shape: ColliderShape,
    dynamic: bool, // false -> collider is static
}

impl Collider {
    pub fn new(shape: ColliderShape, dynamic: bool) -> Self {
        Self { shape, dynamic }
    }

    pub fn get_correction_vec(&self, other: &Self) -> Option<Vector3> {
        self.shape.get_correction_vec(&other.shape)
    }

    pub fn dynamic(&self) -> bool {
        self.dynamic
    }

    pub fn new_box(position: Vector3, dimensions: Vector3, dynamic: bool) -> Self {
        Self {
            shape: ColliderShape::Box(BoxCollider::new(position, dimensions)),
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
