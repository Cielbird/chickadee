use uuid::Uuid;

use crate::component::ComponentId;

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
    #[allow(unused)]
    pub name: String,
    pub components: Vec<ComponentId>,
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            name,
            components: vec![],
        }
    }
}
