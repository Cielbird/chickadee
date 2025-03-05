use super::renderer::Renderer;

pub struct Engine<'a> {
    pub renderer: Renderer<'a>
}

impl<'a> Engine<'a> {
    pub fn new(window: winit::window::Window) -> Self {
        Self {
            renderer: Renderer::new(window)
        }
    }
}
