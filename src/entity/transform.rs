use crate::{transform::Transform, Component, Vector3};

#[derive(Debug, Clone)]
pub struct EntityTransform {
    pub local: Transform,
    // when local transform changes, all the children entities' global transforms need to be updated
    pub dirty: bool,
    pub global: Transform,
}

impl EntityTransform {
    pub fn new() -> Self {
        Self {
            local: Transform::identity(),
            dirty: false,
            global: Transform::identity(),
        }
    }

    #[allow(unused)]
    pub fn local_ref(&self) -> &Transform {
        &self.local
    }

    pub fn global_ref(&self) -> &Transform {
        &self.global
    }

    pub fn translate_global(&mut self, vec: Vector3) {
        self.dirty = true;
        self.local.translate_global(vec)
    }

    pub fn translate_local(&mut self, vec: Vector3) {
        self.dirty = true;
        self.local.translate_local(vec)
    }

    pub fn rotate_euler_global(&mut self, euler: Vector3) {
        self.dirty = true;
        self.local.rotate_euler_global(euler)
    }
}

impl Component for EntityTransform {
    fn on_start(&mut self, _scene: &mut crate::Scene, _context: crate::OnStartContext) {}

    fn on_update(&mut self, _scene: &mut crate::Scene, _context: crate::OnUpdateContext) {}

    fn on_event(&mut self, _scene: &mut crate::Scene, _context: crate::OnEventContext) {}
}
