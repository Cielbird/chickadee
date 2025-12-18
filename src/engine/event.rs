use winit::event::WindowEvent;

use crate::engine::{component::ComponentId, entity::EntityId};

pub struct OnStartContext {
    /// Context: current caller's information
    pub entity: EntityId,
    pub component: ComponentId,
}

pub struct OnUpdateContext {
    /// Context: current caller's information
    pub entity: EntityId,
    pub component: ComponentId,
}

pub struct OnEventContext {
    /// Context: current caller's information
    pub entity: EntityId,
    pub component: ComponentId,
    
    /// Window event
    pub event: WindowEvent,
}
