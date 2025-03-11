use std::any::TypeId;

pub trait Component {
    fn get_type_id(&self) -> TypeId;
}


