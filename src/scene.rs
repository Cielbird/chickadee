use crate::component::{Component, ComponentId, ComponentRef};
use crate::entity::Entity;
use crate::event::{OnEventContext, OnStartContext, OnUpdateContext};
use crate::model::Model;
use crate::{Collider, EntityTransform};
use std::collections::{HashMap, VecDeque};

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
            root,
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
            .add_component(scene.root, EntityTransform::new())
            .unwrap();

        scene
    }

    pub fn get_root(&self) -> EntityId {
        self.root
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
                    let global_transform = transform.global();

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
                    entity: entity_id,
                    component: component_id,
                },
            );
        }
    }

    pub fn on_update(&mut self) {
        // update transforms
        self.update_transforms();

        // do collider logic
        self.collider_pass();

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

        let transform = EntityTransform::new();
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

    // TODO transforms are now components, so why not just put this in their update() function?
    fn update_transforms(&mut self) {
        let mut frontier = VecDeque::new();
        frontier.push_front(self.root);
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

                    let parent = self.get_transform(parent);
                    let parent = parent.read().unwrap();

                    was_dirty = current.dirty;
                    if was_dirty {
                        current.parent = parent.global();
                        current.dirty = false;
                    }
                } else {
                    // entity had no parent: root
                    let mut current = self.get_transform(&next);
                    let mut current = current.write().unwrap();

                    was_dirty = current.dirty;
                    if was_dirty {
                        current.dirty = false;
                    }
                }
                (node.children.clone(), was_dirty)
            };
            if parent_was_dirty {
                for child_id in &children {
                    let mut child = self.get_transform(child_id);
                    let mut child = child.write().unwrap();
                    child.dirty = true;
                }
            }
            for child in children {
                frontier.push_front(child);
            }
        }
    }

    fn collider_pass(&mut self) {
        let colliders = self.get_colliders();
        if colliders.len() < 2 {
            return;
        }

        for a_idx in 0..(colliders.len() - 1) {
            let (a, col_a) = colliders.get(a_idx).unwrap();
            let mut a_trans = self.get_transform(a);
            let mut a_trans = a_trans.write().unwrap();
            let col_a = col_a.read().unwrap();
            let a_dynamic = col_a.dynamic();

            for b_idx in (a_idx + 1)..colliders.len() {
                let (b, col_b) = colliders.get(b_idx).unwrap();
                let mut b_trans = self.get_transform(b);
                let mut b_trans = b_trans.write().unwrap();
                let col_b = col_b.read().unwrap();
                let b_dynamic = col_b.dynamic();

                if !a_dynamic && !b_dynamic {
                    continue;
                }

                let vec = Collider::get_correction_vec(
                    &col_a,
                    &a_trans.global(),
                    &col_b,
                    &b_trans.global(),
                );

                match vec {
                    Some(vec) => {
                        // move transforms
                        if a_dynamic {
                            if b_dynamic {
                                // a and b are both dynamic
                                a_trans.translate_global(vec / 2.);
                                b_trans.translate_global(-vec / 2.);
                            } else {
                                // only a is dynamic
                                a_trans.translate_global(vec);
                            }
                        } else {
                            // only b is dynamic
                            b_trans.translate_global(-vec);
                        }
                    }
                    None => continue,
                }
            }
        }

        // necesary !
        self.update_transforms();
    }

    fn get_colliders(&mut self) -> Vec<(EntityId, ComponentRef<Collider>)> {
        // TODO this could be cached
        let mut colliders = vec![];
        for (id, component) in &self.components {
            if let Ok(collider) = component.clone().downcast::<Collider>() {
                let entity = self.component_entities.get(id).unwrap();
                colliders.push((*entity, collider));
            }
        }

        colliders
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
