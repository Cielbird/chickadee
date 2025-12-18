use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentId {
    id: Uuid,
}

impl ComponentId {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
        }
    }
}
