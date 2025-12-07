use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, Mutex, RwLock, Weak},
};

use crate::engine::component::ComponentRef;

use super::{
    component::Component,
    entity::{Entity, EntityIterator},
    error::*,
};

pub struct Scene {
    root: Arc<RwLock<Entity>>,
}

impl Scene {
    pub fn new() -> Self {
        let root = Arc::new(RwLock::new(Entity::new()));

        Self { root }
    }

    pub fn get_root(&self) -> Arc<RwLock<Entity>> {
        return Arc::clone(&self.root);
    }

    pub fn update(&mut self) {}

    pub fn get_entity_iter(&self) -> EntityIterator {
        EntityIterator::new(self.root.clone())
    }

    pub fn draw_scene(
        &self,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
    ) -> Result<()> {
        Ok(())
    }

    pub fn find_first_component<C: Component>(&self) -> Option<ComponentRef<C>> {
        for entity in self.get_entity_iter() {
            if let Ok(entity) = entity.read() {
                if let Some(component) = entity.find_first_component() {
                    return Some(component);
                }
            }
        }

        None
    }
}
