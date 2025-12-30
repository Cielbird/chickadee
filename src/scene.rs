use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

use crate::component::{Component, ComponentId, ComponentRef};
use crate::entity::{Entity, EntityTransform};
use crate::event::{OnEventContext, OnStartContext, OnUpdateContext};
use crate::model::Model;

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
        let entity_graph = EntityGraph::new();
        let components = HashMap::new();
        let mut entities = HashMap::new();
        entities.insert(entity_graph.root(), Entity::new("root".to_string()));
        let component_entities = HashMap::new();

        let entity_graph = Arc::new(RwLock::new(entity_graph));

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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera_bind_group: &wgpu::BindGroup,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Result<()> {
        // iterate on all components, render renderable components
        for (entity_id, component_ids) in self.entities.clone() {
            for component_id in component_ids.components {
                let component = self
                    .components
                    .get(&component_id)
                    .expect("Component not found, scene corrupted!")
                    .clone();

                if let Ok(mut model) = TryInto::<ComponentRef<Model>>::try_into(component) {
                    let mut model = model.get_mut().unwrap();

                    let transform = self.get_transform_ref(&entity_id).unwrap();

                    model.draw_model(
                        transform.global_ref(),
                        device,
                        queue,
                        render_pass,
                        camera_bind_group,
                        texture_layout,
                    )?;
                }
            }
        }
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
        // update transforms
        self.update_transforms();

        let component_entities = self.component_entities.clone();
        for (comp_id, entity_id) in component_entities {
            let component = self
                .components
                .get(&comp_id)
                .expect("Component not found, scene corrupted!")
                .clone();

            component.on_update(
                self,
                OnUpdateContext {
                    entity: entity_id.clone(),
                    component: comp_id,
                },
            );
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
        let id = graph.add(parent)?;
        self.entities.insert(id.clone(), Entity::new(name));

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
            .unwrap()
            .components
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

    pub fn get_component<C: Component>(&self, entity: &EntityId) -> Option<(ComponentId, ComponentRef<C>)> {
        let entity = self.entities.get(entity)?;
        for comp_id in &entity.components {
            let comp_ref = self.components.get(comp_id).unwrap();
            // Could be optimized with a haspmap
            if let Ok(x) = comp_ref.clone().try_into() {
                return Some((comp_id.clone(), x));
            }
        }
        None
    }

    pub fn get_transform_ref(&self, entity: &EntityId) -> Option<&EntityTransform> {
        let graph = self.entities.get(entity)?;
        Some(&graph.transform)
    }

    pub fn get_transform_mut(&mut self, entity: &EntityId) -> Option<&mut EntityTransform> {
        let entity = self.entities.get_mut(entity)?;
        Some(&mut entity.transform)
    }

    pub fn get_transform_disjoint_mut<const N: usize>(
        &mut self,
        entities: [&EntityId; N],
    ) -> [Option<&mut EntityTransform>; N] {
        let entities = self.entities.get_disjoint_mut(entities);
        let transforms = entities.map(|e| e.map(|e| &mut e.transform));

        transforms
    }

    pub fn get_entity(&self, comp_id: &ComponentId) -> Option<EntityId> {
        self.component_entities.get(comp_id).cloned()
    }

    pub fn parent(&self, child_id: &EntityId) -> Option<EntityId> {
        let graph = self.entity_graph.read().unwrap();
        graph.node(child_id)?.parent()
    }

    fn update_transforms(&mut self) {
        let graph = self.entity_graph.read().unwrap();
        let mut frontier = VecDeque::new();
        frontier.push_front(graph.root());
        loop {
            let next = frontier.pop_back();
            if next.is_none() {
                break;
            }
            let next = next.unwrap();
            let (children, parent_was_dirty) = {
                let node = graph.node(&next).unwrap();
                let was_dirty;
                if let Some(parent) = node.parent() {
                    let [entity, parent] = self.entities.get_disjoint_mut([&next, &parent]);
                    let entity = entity.unwrap();
                    let parent = parent.unwrap();

                    was_dirty = entity.transform.dirty;
                    if was_dirty {
                        entity.transform.global =
                            parent.transform.global.clone() * entity.transform.local.clone();
                        entity.transform.dirty = false;
                    }
                } else {
                    // entity had no parent: root
                    let entity = self.entities.get_mut(&next).unwrap();

                    was_dirty = entity.transform.dirty;
                    if was_dirty {
                        entity.transform.global = entity.transform.local.clone();
                        entity.transform.dirty = false;
                    }
                }
                (node.children().clone(), was_dirty)
            };
            if parent_was_dirty {
                for child_id in &children {
                    let child = self.entities.get_mut(child_id).unwrap();
                    child.transform.dirty = true
                }
            }
            for child in children {
                frontier.push_front(child.clone());
            }
        }
    }
}
