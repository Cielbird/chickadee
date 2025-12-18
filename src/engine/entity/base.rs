use uuid::Uuid;

use crate::engine::{component::ComponentId, transform::Transform};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityId {
    id: Uuid,
}

impl EntityId {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Entity {
    pub transform: Transform,
    pub components: Vec<ComponentId>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            transform: Transform::identity(),
            components: vec![],
        }
    }
}
