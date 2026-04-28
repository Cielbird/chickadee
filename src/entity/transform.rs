use crate::{transform::Transform, Component, Vector3};

/// Component that represents a transform in the entity hierarchy.
#[derive(Debug, Clone)]
pub struct EntityTransform {
    // Transform to go from current to child: T_local
    local: Transform,
    // the global transform of the parent
    parent: Transform,
    // T_global = T_parent * T_local, or identity() for the root
    global: Transform,
    // when global transform changes, this flag is raised to indicate a need to update children
    children_dirty: bool,
}

impl EntityTransform {
    pub fn new() -> Self {
        Self {
            local: Transform::identity(),
            parent: Transform::identity(),
            global: Transform::identity(),
            children_dirty: false,
        }
    }

    #[allow(unused)]
    pub fn local_ref(&self) -> &Transform {
        &self.local
    }

    /// Translate along global axis
    /// where dT is the global translation
    pub fn translate_global(&mut self, vec: Vector3) {
        let inverse_parent = self.parent.inverse();
        self.local = (inverse_parent * Transform::from_translation(vec) * self.parent) * self.local;

        self.update_global();
    }

    /// Translate along this local axis:
    /// T_local = T_local * dT
    /// where dT is the local translation
    pub fn translate_local(&mut self, vec: Vector3) {
        self.local = self.local * Transform::from_translation(vec);

        self.update_global();
    }

    pub fn rotate_euler_local(&mut self, euler: Vector3) {
        self.local = self.local
            * Transform::from_angle_z(euler.z)
            * Transform::from_angle_y(euler.y)
            * Transform::from_angle_x(euler.x);

        self.update_global();
    }

    /// Sets the parents transform, and calculates the global transform,
    /// clearing the global_dirty flag
    pub fn set_parent(&mut self, parent: Transform) {
        self.parent = parent;
        self.update_global();
    }
    
    fn update_global(&mut self) {
        self.global = self.parent * self.local;
        self.children_dirty = true;
    }

    pub fn global(&self) -> Transform {
        self.global
    }

    pub fn children_dirty(&self) -> bool {
        self.children_dirty
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
