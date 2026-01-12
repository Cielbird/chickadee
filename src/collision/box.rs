use cgmath::vec3;

use crate::{collision::shape::ColliderShape, Component, Vector3};

pub struct BoxCollider {
    position: Vector3,
    // half-width, half-height, and half-depth
    dimensions: Vector3,
}

impl BoxCollider {
    pub fn new(position: Vector3, dimensions: Vector3) -> Self {
        Self {
            position,
            dimensions,
        }
    }

    /// Get the collision correction vector for this box with another shape of collider
    pub fn get_correction_vec(&self, other: &ColliderShape) -> Option<Vector3> {
        match other {
            ColliderShape::Box(other) => self.box_correction_vec(other),
        }
    }

    /// Get the collision correction vector for this box if the other one is also a box
    fn box_correction_vec(&self, other: &BoxCollider) -> Option<Vector3> {
        let dx = if self.position.x > other.position.x {
            (other.position.x + other.dimensions.x) - (self.position.x - self.dimensions.x)
        } else {
            (other.position.x - other.dimensions.x) - (self.position.x + self.dimensions.x)
        };
        let dy = if self.position.y > other.position.y {
            (other.position.y + other.dimensions.y) - (self.position.y - self.dimensions.y)
        } else {
            (other.position.y - other.dimensions.y) - (self.position.y + self.dimensions.y)
        };
        let dz = if self.position.z > other.position.z {
            (other.position.z + other.dimensions.z) - (self.position.z - self.dimensions.z)
        } else {
            (other.position.z - other.dimensions.z) - (self.position.z + self.dimensions.z)
        };

        if dx < 0. && dy < 0. && dz < 0. {
            None
        } else {
            Some(vec3(dx, dy, dz))
        }
    }
}

// colliders could be treated seperately
impl Component for BoxCollider {
    fn on_start(&mut self, _scene: &mut crate::Scene, _context: crate::OnStartContext) {}

    fn on_update(&mut self, _scene: &mut crate::Scene, _context: crate::OnUpdateContext) {}

    fn on_event(&mut self, _scene: &mut crate::Scene, _context: crate::OnEventContext) {}
}
