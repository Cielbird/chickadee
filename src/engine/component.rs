use std::{
    any::Any,
    sync::{Arc, RwLock, Weak},
};

use winit::event::WindowEvent;

use super::{entity::Entity, scene::Scene};

pub trait Component: Send + Sync + 'static {
    fn get_entity(&self) -> Option<Arc<RwLock<Entity>>>;
    fn set_entity(&mut self, entity: &Arc<RwLock<Entity>>);

    fn on_start(&mut self, scene: &mut Scene);
    fn on_update(&mut self, scene: &mut Scene);
    fn on_event(&mut self, scene: &mut Scene, event: &WindowEvent);
}
