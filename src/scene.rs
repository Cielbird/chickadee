use std::collections::{HashMap, VecDeque};

use crate::component::{Component, ComponentId, ComponentRef};
use crate::entity::Entity;
use crate::event::{OnEventContext, OnStartContext, OnUpdateContext};
use crate::model::Model;
use crate::EntityTransform;

use super::{component::DynComponentRef, entity::EntityId};

use super::error::*;

pub struct Scene {
    /// Graph of entities
    nodes: HashMap<EntityId, Node>,
    root: EntityId,

    /// Container for all components
    components: HashMap<ComponentId, DynComponentRef>,
    component_entities: HashMap<ComponentId, EntityId>,
}

pub(crate) struct Node {
    #[allow(unused)]
    parent: Option<EntityId>,
    children: Vec<EntityId>,
    pub entity: Entity,
}

impl Scene {
    pub fn new() -> Self {
        let root = EntityId::new();
        let mut nodes = HashMap::new();
        nodes.insert(
            root.clone(),
            Node {
                parent: None,
                children: vec![],
                entity: Entity::new("root".to_string()),
            },
        );

        let components = HashMap::new();
        let component_entities = HashMap::new();

        let mut scene = Self {
            nodes,
            root,
            components,
            component_entities,
        };

        scene
            .add_component(scene.root.clone(), EntityTransform::new())
            .unwrap();

        scene
    }

    pub fn get_root(&self) -> EntityId {
        self.root.clone()
    }

    pub fn get_transform(&self, entity_id: &EntityId) -> ComponentRef<EntityTransform> {
        let (_, transform) = self
            .get_component::<EntityTransform>(entity_id)
            .expect("All entities must have transforms!");

        transform
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
        for entity_id in self.nodes.keys() {
            let entity = &self.nodes.get(&entity_id).unwrap().entity;

            for component_id in &entity.components {
                let component = self
                    .components
                    .get(&component_id)
                    .expect("Component not found, scene corrupted!")
                    .clone();

                if let Ok(mut model) = TryInto::<ComponentRef<Model>>::try_into(component) {
                    let mut model = model.write().unwrap();

                    let transform = self.get_transform(&entity_id);
                    let transform = transform.read().unwrap();
                    let global_transform = transform.global.clone();

                    model.draw_model(
                        &global_transform,
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
        for (component_id, entity_id) in self.component_entities.clone() {
            let component = self
                .components
                .get(&component_id)
                .expect("Component not found, scene corrupted!")
                .clone();
            let _ = component.try_on_start(
                self,
                OnStartContext {
                    entity: entity_id.clone(),
                    component: component_id,
                },
            );
        }
    }

    pub fn on_update(&mut self) {
        // update transforms
        self.update_transforms();

        for (component_id, entity_id) in self.component_entities.clone() {
            let component = self
                .components
                .get(&component_id)
                .expect("Component not found, scene corrupted!")
                .clone();

            let _ = component.try_on_update(
                self,
                OnUpdateContext {
                    entity: entity_id.clone(),
                    component: component_id,
                },
            );
        }
    }

    pub fn on_event(&mut self, event: &winit::event::WindowEvent) {
        for (component_id, entity_id) in self.component_entities.clone() {
            let component = self
                .components
                .get(&component_id)
                .expect("Component not found, scene corrupted!")
                .clone();
            let _ = component.try_on_event(
                self,
                OnEventContext {
                    entity: entity_id.clone(),
                    component: component_id,
                    event: event.into(),
                },
            );
        }
    }

    pub fn add_entity(&mut self, parent: EntityId, name: String) -> Result<EntityId> {
        let id = EntityId::new();
        let new_node = Node {
            parent: Some(parent.clone()),
            children: vec![],
            entity: Entity::new(name),
        };

        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(id.clone());
        }

        self.nodes.insert(id.clone(), new_node);

        let transform = EntityTransform::new();
        self.add_component(id.clone(), transform)?;

        Ok(id)
    }

    pub fn add_component<C: Component>(&mut self, entity: EntityId, component: C) -> Result<()> {
        if !self.nodes.contains_key(&entity) {
            return Err(Error::Other("Entity not found!".to_string()));
        }

        let component_ref = DynComponentRef::new(component);
        let component_id = ComponentId::new();
        self.components.insert(component_id.clone(), component_ref);

        let entity_node = self.nodes.get_mut(&entity).unwrap();
        entity_node.entity.components.push(component_id.clone());

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

    pub fn get_component<C: Component>(
        &self,
        entity: &EntityId,
    ) -> Option<(ComponentId, ComponentRef<C>)> {
        let entity = &self.nodes.get(entity)?.entity;

        for comp_id in &entity.components {
            let comp_ref = self.components.get(comp_id).unwrap();
            // Could be optimized with a haspmap
            if let Ok(x) = comp_ref.clone().try_into() {
                return Some((comp_id.clone(), x));
            }
        }
        None
    }

    pub fn get_entity(&self, comp_id: &ComponentId) -> Option<EntityId> {
        self.component_entities.get(comp_id).cloned()
    }

    pub fn parent(&self, child_id: &EntityId) -> Option<EntityId> {
        self.nodes.get(child_id)?.parent.clone()
    }

    fn update_transforms(&mut self) {
        let mut frontier = VecDeque::new();
        frontier.push_front(self.root.clone());
        loop {
            let next = frontier.pop_back();
            if next.is_none() {
                break;
            }
            let next = next.unwrap();
            let (children, parent_was_dirty) = {
                let node = self.nodes.get(&next).unwrap();
                let was_dirty;
                if let Some(parent) = &node.parent {
                    let mut current = self.get_transform(&next);
                    let mut current = current.write().unwrap();

                    let parent = self.get_transform(&parent);
                    let parent = parent.read().unwrap();

                    was_dirty = current.dirty;
                    if was_dirty {
                        current.global = parent.global.clone() * current.local.clone();
                        current.dirty = false;
                    }
                } else {
                    // entity had no parent: root
                    let mut current = self.get_transform(&next);
                    let mut current = current.write().unwrap();

                    was_dirty = current.dirty;
                    if was_dirty {
                        current.global = current.local.clone();
                        current.dirty = false;
                    }
                }
                (node.children.clone(), was_dirty)
            };
            if parent_was_dirty {
                for child_id in &children {
                    let mut child = self.get_transform(&child_id);
                    let mut child = child.write().unwrap();
                    child.dirty = true;
                }
            }
            for child in children {
                frontier.push_front(child.clone());
            }
        }
    }
}
