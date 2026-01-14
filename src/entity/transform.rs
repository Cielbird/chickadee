use crate::{transform::Transform, Component, Vector3};

/// Component that represents a transform in the entity hierarchy.
#[derive(Debug, Clone)]
pub struct EntityTransform {
    // Transform to go from current to child: T_local
    pub local: Transform,
    // when local transform changes, all the children entities' global transforms need to be updated
    pub dirty: bool,
    // the transform of the parent: T_global = T_parent * T_local, or identity() for the root
    pub parent: Transform,
}

impl EntityTransform {
    pub fn new() -> Self {
        Self {
            local: Transform::identity(),
            dirty: false,
            parent: Transform::identity(),
        }
    }

    #[allow(unused)]
    pub fn local_ref(&self) -> &Transform {
        &self.local
    }

    /// Translate along global axis
    /// where dT is the global translation
    pub fn translate_global(&mut self, vec: Vector3) {
        self.dirty = true;

        let inverse_parent = self.parent.inverse();

        self.local = (inverse_parent * Transform::from_translation(vec) * self.parent) * self.local;
    }

    /// Translate along this local axis:
    /// T_local = T_local * dT
    /// where dT is the local translation
    pub fn translate_local(&mut self, vec: Vector3) {
        self.dirty = true;
        self.local = self.local * Transform::from_translation(vec)
    }

    pub fn rotate_euler_local(&mut self, euler: Vector3) {
        self.dirty = true;
        self.local = self.local
            * Transform::from_angle_z(euler.z)
            * Transform::from_angle_y(euler.y)
            * Transform::from_angle_x(euler.x)
    }

    pub fn global(&self) -> Transform {
        self.parent * self.local
    }
}

impl Component for EntityTransform {
    fn on_start(&mut self, _scene: &mut crate::Scene, _context: crate::OnStartContext) {}

    fn on_update(&mut self, _scene: &mut crate::Scene, _context: crate::OnUpdateContext) {}

    fn on_event(&mut self, _scene: &mut crate::Scene, _context: crate::OnEventContext) {}
}

impl Default for EntityTransform {
    fn default() -> Self {
        Self::new()
    }
}
