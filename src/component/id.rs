use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentId {
    id: Uuid,
}

#[allow(clippy::new_without_default)]
impl ComponentId {
    pub fn new() -> Self {
        // TODO replace with faster id
        Self { id: Uuid::new_v4() }
    }
}
