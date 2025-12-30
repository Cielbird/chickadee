use crate::engine::get_engine;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

/// Handles running the engine for the winit window
pub struct EngineHandler {}

impl<'a> EngineHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl ApplicationHandler for EngineHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_attribs = Window::default_attributes().with_title("Hello!");
        let window = event_loop.create_window(win_attribs).unwrap();
        let engine = get_engine();
        let mut engine = engine.write().unwrap();
        engine.set_window(window);
        engine.on_start();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let engine = get_engine();
        let engine = engine.read().unwrap();
        let renderer = engine.renderer.as_ref().unwrap();
        let mut renderer = renderer.lock().unwrap();
        let window = renderer.window();

        if window.id() == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    renderer.render().unwrap();

                    engine.on_update();
                }
                _ => {
                    engine.on_event(&event);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let engine = get_engine();
        let engine = engine.read().unwrap();
        let window = engine.get_window();
        window.request_redraw();
    }
}
