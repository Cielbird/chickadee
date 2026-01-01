use crate::{transform::Transform, Component};

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

    pub fn move_global(&mut self, vec: cgmath::Vector3<f32>) {
        self.dirty = true;
        self.local.move_global(vec)
    }

    pub fn move_local(&mut self, vec: cgmath::Vector3<f32>) {
        self.dirty = true;
        self.local.move_local(vec)
    }

    pub fn rotate_euler_global(&mut self, euler: cgmath::Vector3<f32>) {
        self.dirty = true;
        self.local.rotate_euler_global(euler)
    }
}

impl Component for EntityTransform {
    fn on_start(&mut self, scene: &mut crate::Scene, context: crate::OnStartContext) {}

    fn on_update(&mut self, scene: &mut crate::Scene, context: crate::OnUpdateContext) {}

    fn on_event(&mut self, scene: &mut crate::Scene, context: crate::OnEventContext) {}
}
