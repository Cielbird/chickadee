use std::sync::atomic::AtomicU32;

use crate::component::ComponentId;

static NEXT_ENTITY_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    id: u32,
}

#[allow(clippy::new_without_default)]
impl EntityId {
    pub fn new() -> Self {
        let id = NEXT_ENTITY_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self { id }
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
