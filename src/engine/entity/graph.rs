use super::base::EntityId;
use crate::engine::error::*;
use std::collections::HashMap;

/// Module for the entity graph
pub struct EntityGraph {
    root: EntityId,
    nodes: HashMap<EntityId, Node>,
}

pub struct Node {
    #[allow(unused)]
    parent: Option<EntityId>,
    children: Vec<EntityId>,
    // TODO put entity data here?
}

impl EntityGraph {
    pub fn new() -> Self {
        let root = EntityId::new();
        let mut nodes = HashMap::new();
        nodes.insert(root.clone(), Node {
            parent: None,
            children: vec![],
        });
        Self {
            root,
            nodes,
        }
    }

    pub fn root(&self) -> EntityId {
        self.root.clone()
    }

    pub fn add(&mut self, parent: EntityId) -> Result<EntityId> {
        let id = EntityId::new();
        let new_node = Node {
            parent: Some(parent.clone()),
            children: vec![],
        };

        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(id.clone());
        }

        self.nodes.insert(id.clone(), new_node);

        Ok(id)
    }

    pub fn contains(&self, entity: &EntityId) -> bool {
        self.nodes.contains_key(entity)
    }

    pub fn node(&self, id: &EntityId) -> Option<&Node> {
        self.nodes.get(id)
    }
}

impl Node {
    pub fn children(&self) -> &Vec<EntityId> {
        &self.children
    }

    pub fn parent(&self) -> Option<EntityId> {
        self.parent.clone()
    }
}
