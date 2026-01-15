use std::time::Duration;

use crate::{component::ComponentId, entity::EntityId};

pub struct OnStartContext {
    /// Context: current caller's information
    pub entity: EntityId,
    pub component: ComponentId,
}

pub struct OnUpdateContext {
    /// Context: current caller's information
    pub entity: EntityId,
    pub component: ComponentId,
    /// time since last OnUpdate call
    pub delta_time: Duration,
}

pub struct OnEventContext {
    /// Context: current caller's information
    pub entity: EntityId,
    pub component: ComponentId,

    /// Window event
    pub event: WindowEvent,
}

pub enum WindowEvent {
    KeyboardInput { event: KeyEvent },
    CursorMoved { position_x: f32, position_y: f32 },
    Other, // Any other irrelevant events
}

pub struct KeyEvent {
    pub is_pressed: bool,
    pub key: KeyCode,
}

pub type KeyCode = winit::keyboard::KeyCode;

impl From<&winit::event::WindowEvent> for WindowEvent {
    fn from(event: &winit::event::WindowEvent) -> WindowEvent {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
                    WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            is_pressed: event.state.is_pressed(),
                            key: keycode,
                        },
                    }
                } else {
                    todo!()
                }
            }
            winit::event::WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => WindowEvent::CursorMoved {
                position_x: position.x as f32,
                position_y: position.y as f32,
            },

            // All other events are unprocessed by the engine
            _ => WindowEvent::Other,
        }
    }
}
