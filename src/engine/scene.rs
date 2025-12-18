use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use crate::engine::component::{Component, ComponentId, ComponentRef};
use crate::engine::entity::Entity;
use crate::engine::event::{OnEventContext, OnStartContext, OnUpdateContext};
use crate::engine::transform::Transform;

use super::{component::DynComponentRef, entity::EntityId};

use super::{entity::EntityGraph, error::*};

pub struct Scene {
    /// Graph of entities
    entity_graph: Arc<RwLock<EntityGraph>>,

    /// Container for all components
    components: HashMap<ComponentId, DynComponentRef>,
    entities: HashMap<EntityId, Entity>,
    component_entities: HashMap<ComponentId, EntityId>,
}

impl Scene {
    pub fn new() -> Self {
        let entity_graph = Arc::new(RwLock::new(EntityGraph::new()));
        let components = HashMap::new();
        let entities = HashMap::new();
        let component_entities = HashMap::new();

        Self {
            entity_graph,
            components,
            entities,
            component_entities,
        }
    }

    pub fn get_root(&self) -> EntityId {
        let graph = self.entity_graph.read().unwrap();
        return graph.root();
    }

    pub fn draw_scene(
        &self,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
    ) -> Result<()> {
        Ok(())
    }

    pub fn on_start(&mut self) {
        for (entity_id, component_ids) in self.entities.clone() {
            for component_id in component_ids.components {
                let component = self
                    .components
                    .get(&component_id)
                    .expect("Component not found, scene corrupted!")
                    .clone();
                component.on_start(
                    self,
                    OnStartContext {
                        entity: entity_id.clone(),
                        component: component_id,
                    },
                );
            }
        }
    }

    pub fn on_update(&mut self) {
        for (entity_id, component_ids) in self.entities.clone() {
            for component_id in component_ids.components {
                let component = self
                    .components
                    .get(&component_id)
                    .expect("Component not found, scene corrupted!")
                    .clone();
                component.on_update(
                    self,
                    OnUpdateContext {
                        entity: entity_id.clone(),
                        component: component_id,
                    },
                );
            }
        }
    }

    pub fn on_event(&mut self, event: &winit::event::WindowEvent) {
        for (entity_id, component_ids) in self.entities.clone() {
            for component_id in component_ids.components {
                let component = self
                    .components
                    .get(&component_id)
                    .expect("Component not found, scene corrupted!")
                    .clone();
                component.on_event(
                    self,
                    OnEventContext {
                        entity: entity_id.clone(),
                        component: component_id,
                        event: event.clone(),
                    },
                );
            }
        }
    }

    pub fn add_entity(&mut self, parent: EntityId, name: String) -> Result<EntityId> {
        let mut graph = self.entity_graph.write().unwrap();
        let id = graph.add(parent, name)?;
        self.entities.insert(id.clone(), Entity::new());
        
        Ok(id)
    }

    pub fn add_component<C: Component>(&mut self, entity: EntityId, component: C) -> Result<()> {
        let graph = self.entity_graph.read().unwrap();
        if !graph.contains(&entity) {
            return Err(Error::Other("Entity not found!".to_string()));
        }
        let component_ref = DynComponentRef::new(component);
        let component_id = ComponentId::new();
        self.components.insert(component_id.clone(), component_ref);
        self.entities
            .get_mut(&entity)
            .unwrap().components
            .push(component_id.clone());

        self.component_entities.insert(component_id, entity);

        Ok(())
    }

    pub fn find_first_component<C: Component>(&self) -> Option<(ComponentId, ComponentRef<C>)> {
        for (comp_id, comp_ref) in &self.components {
            // Could be optimized with a haspmap
            if let Ok(x) = comp_ref.clone().try_into() {
                return Some((comp_id.clone(), x));
            }
        }

        None
    }

    pub fn get_tranform_ref(&self, entity: &EntityId) -> Option<&Transform> {
        Some(&self.entities.get(&entity)?.transform)
    }
    
    pub fn get_tranform_mut(&mut self, entity: &EntityId) -> Option<&mut Transform> {
        Some(&mut self.entities.get_mut(&entity)?.transform)
    }
    
    pub(crate) fn get_entity(&self, comp_id: &ComponentId) -> Option<&EntityId> {
        self.component_entities.get(comp_id)
    }
}
