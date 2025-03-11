use uuid::Uuid;


pub type EntityId = Uuid;

pub trait EntityIdExt {
    fn new() -> EntityId;
}

impl EntityIdExt for EntityId {
    fn new() -> Self {
        Self::new_v4() as EntityId
    }
}
