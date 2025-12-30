use uuid::Uuid;

use crate::{component::ComponentId, transform::Transform};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityId {
    id: Uuid,
}

impl EntityId {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Entity {
    #[allow(unused)]
    pub name: String,
    pub components: Vec<ComponentId>,
    pub transform: EntityTransform,
}

#[derive(Debug, Clone)]
pub struct EntityTransform {
    pub local: Transform,
    // when local transform changes, all the children entities' global transforms need to be updated
    pub dirty: bool,
    pub global: Transform,
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            name,
            components: vec![],
            transform: EntityTransform::new(),
        }
    }
}

impl EntityTransform {
    fn new() -> Self {
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
