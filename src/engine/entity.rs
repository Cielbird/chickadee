use std::{any::{Any, TypeId}, sync::{Arc, RwLock, Weak}};

use winit::event::WindowEvent;

use super::{component::Component, scene::Scene, transform::{self, Transform}};


pub struct Entity {
    components: Vec<Arc<dyn Any + Send + Sync>>,
    transform: Transform,
    parent: Option<Weak<RwLock<Entity>>>,
    children: Vec<Arc<RwLock<Entity>>>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            components: vec![],
            transform: Transform::zero(),
            parent: None,
            children: vec![],
        }
    }

    pub fn get_tranform(&self) -> &Transform {
        &self.transform
    }

    pub fn add_child(parent: &Arc<RwLock<Entity>>) -> Arc<RwLock<Entity>> {
        let mut child = Self::new();
        child.parent = Some(Arc::downgrade(parent));

        let child = Arc::new(RwLock::new(child));
        parent.write().unwrap().children.push(Arc::clone(&child));

        child
    }

    pub fn add_component<C: Component>(entity: &Arc<RwLock<Entity>>, component: C) {
        let c = Arc::new(RwLock::new(component));
        c.write().unwrap().set_entity(entity);
        entity.write().unwrap().components.push(c);
    }

    pub fn find_first_component<C>(&self) -> Option<Arc<RwLock<C>>> 
        where C: Component 
    {
        for c in &self.components {
            if let Ok(c) = c.clone().downcast::<RwLock<C>>() {
                return Some(c);
            } else {
                println!("Failed to downcast");
            }
        }
        
        None
    }

    pub fn on_start(&mut self, scene: &mut Scene) {
        for c in &self.components {
            if let Some(c) = c.downcast_ref::<Arc<RwLock<dyn Component>>>() {
                c.write().unwrap().on_start(scene);
            } else {
                println!("Failed to downcast to component");
            }
        }
    }

    pub fn on_update(&mut self, scene: &mut Scene) {
        for c in &self.components {
            if let Some(c) = c.downcast_ref::<Arc<RwLock<dyn Component>>>() {
                c.write().unwrap().on_update(scene);
            } else {
                println!("Failed to downcast to component");
            }
        }
    }

    pub fn on_event(&mut self, scene: &mut Scene, event: &WindowEvent) {
        for c in &self.components {
            if let Some(c) = c.downcast_ref::<Arc<RwLock<dyn Component>>>() {
                c.write().unwrap().on_event(scene, event);
            } else {
                println!("Failed to downcast to component");
            }
        }
    }
}

pub struct EntityIterator {
    stack: Vec<Arc<RwLock<Entity>>>,  // nodes to visit
}

impl EntityIterator {
    pub fn new(root: Arc<RwLock<Entity>>) -> Self {
        let mut stack = Vec::new();
        stack.push(root);  // Start with the root node
        EntityIterator { stack }
    }
}

impl Iterator for EntityIterator {
    type Item = Arc<RwLock<Entity>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity) = self.stack.pop() {
            // Push children to stack to visit later
            for child in entity.read().unwrap().children.clone() {
                self.stack.push(child);
            }
            Some(entity)
        } else {
            None  // No more nodes to visit
        }
    }
}