use std::{
    any::{Any, TypeId},
    collections::HashMap,
    vec,
};

use super::{component::Component, entity::EntityId};

pub trait ComponentStorable: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct ComponentStorage<T> {
    components: Vec<T>,
    entity_map: HashMap<EntityId, usize>,
}

impl<T: 'static> ComponentStorable for ComponentStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<C: Component + 'static> ComponentStorage<C> {
    pub fn new() -> ComponentStorage<C> {
        Self {
            components: vec![],
            entity_map: HashMap::new(),
        }
    }

    pub fn push(&mut self, entity: EntityId, component: C) {
        self.components.push(component);
        self.entity_map.insert(entity, self.components.len());
    }
}
