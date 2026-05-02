use crate::component::{Component, ComponentId, ComponentStore};
use crate::entity::transform::TransformComponent;
use crate::entity::Entity;
use crate::event::{OnEventContext, OnStartContext, OnUpdateContext};
use crate::model::Model;
use crate::{Camera, Collider, CollisionArena};
use std::collections::hash_map::Keys;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use super::entity::EntityId;

use super::error::*;

pub struct Scene {
    /// Graph of entities
    nodes: HashMap<EntityId, Node>,
    root: EntityId,

    /// Container for all components
    component_store: ComponentStore,
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

        let component_store = ComponentStore::new();
        let component_entities = HashMap::new();
        let collision = CollisionArena::new();

        let mut scene = Self {
            nodes,
            root,
            component_store,
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

    pub fn get_transform(&self, entity_id: &EntityId) -> ComponentId {
        self.get_first_component_id_from_entity::<TransformComponent>(entity_id)
            .expect("All entities must have transforms!") // TODO throw error
    }

    pub fn get_mut_transform(&mut self, entity_id: &EntityId) -> &mut TransformComponent {
        let id = &self.get_transform(entity_id);
        self.get_mut_component(id)
            .expect("All entities must have transforms!") // TODO throw error
    }

    pub fn on_start(&mut self) {
        for (component_id, entity_id) in self.component_entities.clone() {
            // swap component out
            let mut component = self
                .component_store
                .swap(&component_id, None)
                .expect("Component not found, scene corrupted!");

            // run update
            let _ = component.try_on_start(
                self,
                OnStartContext {
                    entity: entity_id,
                    component: component_id.clone(),
                },
            );

            // swap component back in
            if self
                .component_store
                .swap(&component_id, Some(component))
                .is_some()
            {
                panic!("Component duplicate found, scene corrupted!");
            }
        }
    }

    pub fn on_update(&mut self, delta_time: Duration) {
        // do collider logic
        self.collision.collider_pass(&mut self.component_store);

        // update transforms
        self.update_transforms();

        for (component_id, entity_id) in self.component_entities.clone() {
            // swap component out
            let mut component = self
                .component_store
                .swap(&component_id, None)
                .expect("Component not found, scene corrupted!");

            // run update
            let _ = component.try_on_update(
                self,
                OnUpdateContext {
                    entity: entity_id,
                    component: component_id.clone(),
                    delta_time,
                },
            );

            // swap component back in
            if self
                .component_store
                .swap(&component_id, Some(component))
                .is_some()
            {
                panic!("Component duplicate found, scene corrupted!");
            }
        }

        // clear transform dirty flags
        self.clear_dirty_transforms();
    }

    pub fn on_event(&mut self, event: &winit::event::WindowEvent) {
        for (component_id, entity_id) in self.component_entities.clone() {
            // swap component out
            let mut component = self
                .component_store
                .swap(&component_id, None)
                .expect("Component not found, scene corrupted!");

            // run update
            let _ = component.try_on_event(
                self,
                OnEventContext {
                    entity: entity_id,
                    component: component_id.clone(),
                    event: event.into(),
                },
            );

            // swap component back in
            if self
                .component_store
                .swap(&component_id, Some(component))
                .is_some()
            {
                panic!("Component duplicate found, scene corrupted!");
            }
        }
    }

    pub fn add_component<C: Component>(
        &mut self,
        entity: EntityId,
        component: C,
    ) -> Result<ComponentId> {
        if !self.nodes.contains_key(&entity) {
            return Err(Error::Other("Entity not found!".to_string()));
        }

        let id = self.component_store.insert(component).unwrap();

        self.component_entities.insert(id.clone(), entity);

        let entity_node = self.nodes.get_mut(&entity).unwrap();
        entity_node.entity.components.push(id.clone());
        Ok(id)
    }

    pub fn get_mut_component<C: Component>(&mut self, id: &ComponentId) -> Option<&mut C> {
        self.component_store.get_mut(id)
    }

    pub fn get_ref_component<C: Component>(&self, id: &ComponentId) -> Option<&C> {
        self.component_store.get_ref(id)
    }

    pub fn get_mut_first_component<C: Component>(&mut self) -> Option<&mut C> {
        self.component_store.get_mut_first()
    }

    pub fn get_ref_first_component<C: Component>(&self) -> Option<&C> {
        self.component_store.get_ref_first()
    }

    pub fn get_id_first_component<C: Component>(&self) -> Option<ComponentId> {
        self.component_store.get_id_first::<C>()
    }

    pub fn get_first_component_id_from_entity<C: Component>(&self, entity: &EntityId) -> Option<ComponentId> {
        let entity = &self.nodes.get(entity)?.entity;
        for c in &entity.components {
            if let Some(_) = self.get_ref_component::<C>(&c) {
                return Some(c.clone());
            }
        }
        None
    }

    pub fn get_mut_disjoint_2<C1: Component, C2: Component>(
        &mut self,
        ids: [&ComponentId; 2],
    ) -> (Option<&mut C1>, Option<&mut C2>) {
        self.component_store.get_mut_disjoint_2(ids)
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

    pub fn add_collider(&mut self, entity: EntityId, collider: Collider) {
        let transform = self.get_transform(&entity);
        self.collision.add_collider(entity, collider, transform)
    }

    pub fn get_component_entity(&self, comp_id: &ComponentId) -> Option<EntityId> {
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
                let mut new_global = None;

                let current = self.get_mut_transform(&next);

                let dirty = current.is_dirty();
                if dirty {
                    new_global = Some(current.global());
                }

                let node = self.nodes.get(&next).unwrap();
                (node.children.clone(), new_global)
            };
            for child in children {
                if let Some(new_global) = new_global {
                    let child = self.get_mut_transform(&child);
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

            let current = self.get_mut_transform(&next);

            let dirty = current.is_dirty();
            // only if transform is dirty so are its children
            if dirty {
                current.clear_dirty();
                let node = self.nodes.get(&next).unwrap();
                for child in &node.children {
                    frontier.push_front(child.clone());
                }
            }
        }
    }

    pub(crate) fn entities(&self) -> Vec<EntityId> {
        self.nodes.keys().cloned().collect::<Vec<_>>()
    }

    pub(crate) fn get_entity(&self, entity: &EntityId) -> Option<&Entity> {
        self.nodes.get(entity).map(|x| &x.entity)
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
