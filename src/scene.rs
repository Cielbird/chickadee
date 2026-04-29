use crate::component::{Component, ComponentId, ComponentRef};
use crate::entity::Entity;
use crate::event::{OnEventContext, OnStartContext, OnUpdateContext};
use crate::model::Model;
use crate::{Camera, Collider, CollisionArena, TransformComponent};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use super::{component::DynComponentRef, entity::EntityId};

use super::error::*;

pub struct Scene {
    /// Graph of entities
    nodes: HashMap<EntityId, Node>,
    root: EntityId,

    /// Container for all components
    components: HashMap<ComponentId, DynComponentRef>,
    component_entities: HashMap<ComponentId, EntityId>,

    collision: CollisionArena,
}

pub(crate) struct Node {
    parent: Option<EntityId>,
    children: Vec<EntityId>,
    pub entity: Entity,
}

impl Scene {
    pub fn new() -> Self {
        let root = EntityId::new();
        let mut nodes = HashMap::new();
        nodes.insert(
            root,
            Node {
                parent: None,
                children: vec![],
                entity: Entity::new("root".to_string()),
            },
        );

        let components = HashMap::new();
        let component_entities = HashMap::new();
        let collision = CollisionArena::new();

        let mut scene = Self {
            nodes,
            root,
            components,
            component_entities,
            collision,
        };

        scene
            .add_component(scene.root, TransformComponent::new())
            .unwrap();

        scene
    }

    pub fn get_root(&self) -> EntityId {
        self.root
    }

    pub fn get_transform(&self, entity_id: &EntityId) -> ComponentRef<TransformComponent> {
        let (_, transform) = self
            .get_component::<TransformComponent>(entity_id)
            .expect("All entities must have transforms!"); // TODO throw error

        transform
    }

    pub fn draw_scene(
        &self,
        render_pass: &mut wgpu::RenderPass,
        camera: &Camera,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // TODO i'd love to find a way to nuke these arguments
        camera_bind_group: &wgpu::BindGroup,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Result<()> {
        // iterate on all components, render renderable components
        for entity_id in self.nodes.keys() {
            let entity = &self.nodes.get(entity_id).unwrap().entity;

            for component_id in &entity.components {
                let component = self
                    .components
                    .get(component_id)
                    .expect("Component not found, scene corrupted!")
                    .clone();

                if let Ok(mut model) = TryInto::<ComponentRef<Model>>::try_into(component) {
                    let mut model = model.write().unwrap();

                    let transform = self.get_transform(entity_id);
                    let transform = transform.read().unwrap();

                    model.draw_model(
                        &transform,
                        camera,
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
                    entity: entity_id,
                    component: component_id,
                },
            );
        }
    }

    pub fn on_update(&mut self, delta_time: Duration) {
        // do collider logic
        self.collision.collider_pass();

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
                    entity: entity_id,
                    component: component_id,
                    delta_time,
                },
            );
        }

        // clear transform dirty flags
        self.clear_dirty_transforms();
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
                    entity: entity_id,
                    component: component_id,
                    event: event.into(),
                },
            );
        }
    }

    pub fn add_entity(&mut self, parent: EntityId, name: String) -> Result<EntityId> {
        let id = EntityId::new();
        let new_node = Node {
            parent: Some(parent),
            children: vec![],
            entity: Entity::new(name),
        };

        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(id);
        }

        self.nodes.insert(id, new_node);

        let transform = TransformComponent::new();
        self.add_component(id, transform)?;

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

    pub fn add_collider(&mut self, entity: EntityId, collider: Collider) {
        let transform = self.get_transform(&entity);
        self.collision.add_collider(entity, collider, transform)
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
        self.nodes.get(child_id)?.parent
    }

    fn update_transforms(&mut self) {
        let mut frontier = VecDeque::new();
        frontier.push_front(self.root);
        // traverse transform tree from root
        loop {
            let next = frontier.pop_back();
            if next.is_none() {
                break;
            }
            let next = next.unwrap();
            let (children, new_global) = {
                let node = self.nodes.get(&next).unwrap();
                let mut new_global = None;

                let current = self.get_transform(&next);
                let current = current.read().unwrap();

                let dirty = current.is_dirty();
                if dirty {
                    new_global = Some(current.global());
                }

                (node.children.clone(), new_global)
            };
            for child in children {
                if let Some(new_global) = new_global {
                    let mut child = self.get_transform(&child);
                    let mut child = child.write().unwrap();
                    child.set_parent(new_global);
                }
                frontier.push_front(child);
            }
        }
    }

    /// Clear the dirty flags on each transform
    fn clear_dirty_transforms(&mut self) {
        let mut frontier = VecDeque::new();
        frontier.push_front(self.root);
        // traverse transform tree from root
        loop {
            let next = frontier.pop_back();
            if next.is_none() {
                break;
            }
            let next = next.unwrap();

            let node = self.nodes.get(&next).unwrap();

            let mut current = self.get_transform(&next);
            let mut current = current.write().unwrap();

            let dirty = current.is_dirty();
            // only if transform is dirty so are its children
            if dirty {
                current.clear_dirty();
                for child in &node.children {
                    frontier.push_front(child.clone());
                }
            }
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
