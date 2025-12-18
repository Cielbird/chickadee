use std::{collections::HashMap};
use crate::engine::error::*;
use super::base::EntityId;

/// Module for the entity graph
pub struct EntityGraph {
    root: EntityId,
    nodes: HashMap<EntityId, Node>,
}

struct Node {
    name: String,
    parent: Option<EntityId>,
    children: Vec<EntityId>,
}

impl EntityGraph {
    pub fn new() -> Self {
        Self {
            root: EntityId::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn root(&self) -> EntityId {
        self.root.clone()
    }

    pub fn add(&mut self, parent: EntityId, name: String) -> Result<EntityId> {
        let id = EntityId::new();
        let new_node = Node { name, parent: Some(parent.clone()), children: vec![] };

        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(id.clone());
        }

        self.nodes.insert(id.clone(), new_node);

        Ok(id)
    }

    pub fn contains(&self, entity: &EntityId) -> bool {
        self.nodes.contains_key(entity)
    }
}
