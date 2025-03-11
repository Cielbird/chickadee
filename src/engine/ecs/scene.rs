use std::{
    any::TypeId, 
    collections::HashMap, 
    sync::{Arc, Mutex}
};

use super::{
    component::Component, entity::Entity, error::*
};

pub struct Scene {
    root: Arc<Mutex<Entity>>,
}

impl Scene {
    pub fn new() -> Self {
        let root = Entity::new();

        Self {
            root
        }
    } 

    pub fn add_component<C>(&mut self, entity: EntityId, component: C) -> Result<()> 
        where C: Component + 'static,
    {
        if !self.entities.contains(&entity) {
            return Err(Error::Other(format!("The entity {} does not exist", entity)));
        }
        let type_id = TypeId::of::<C>();
        let component_store_box: &mut Box<dyn ComponentStorable>;

        if !self.component_stores.contains_key(&type_id) {
            let new_storage = ComponentStorage::<C>::new();
            let storable_box: Box<dyn ComponentStorable> = Box::new(new_storage);
            self.component_stores.insert(type_id, storable_box);
        }

        component_store_box = self.component_stores.get_mut(&type_id)
            .ok_or(
                Error::Other(format!("Couldn't find corresponding store: {:?}", type_id))
            )?;

        if let Some(component_store) = component_store_box.as_any_mut().downcast_mut::<ComponentStorage<C>>() {
            component_store.push(entity, component);
        } else {
            return Err(Error::Other(format!("Component storage does not hold the storage of the type it should. type id was {:?}", type_id)));
        }
        
        Ok(())
    }

    pub fn add_new_entity(&mut self) -> EntityId {
        let entity = EntityId::new();
        self.entities.push(entity);

        entity
    }

    pub fn draw_scene(
        &self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup
    ) -> Result<()> {
        Ok(())
    }
}
