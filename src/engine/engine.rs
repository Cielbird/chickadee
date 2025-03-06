use std::sync::Arc;

use winit::window::Window;

use super::renderer::Renderer;

pub struct Engine<'a> {
    pub renderer: Renderer<'a>
}

impl<'a> Engine<'a> {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            renderer: Renderer::new(window)
        }
    }
}
