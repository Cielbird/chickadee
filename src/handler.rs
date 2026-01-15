use crate::engine::get_engine;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use std::time::{Duration, Instant};

/// Handles running the engine for the winit window
pub struct EngineHandler {
    last_render: Instant,
    last_update: Instant,
    render_dt: Duration,
    update_dt: Duration,
}

impl EngineHandler {
    pub fn new() -> Self {
        Self {
            last_render: Instant::now(),
            last_update: Instant::now(),
            render_dt: Duration::from_secs_f64(1.0 / 60.0),
            update_dt: Duration::from_secs_f64(1.0 / 60.0),
        }
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
                }
                _ => {
                    engine.on_event(&event);
                }
            }
        }
    }
    
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        let update_dt_measure = now.duration_since(self.last_update);
        if update_dt_measure >= self.update_dt {
            let engine = get_engine();
            let engine = engine.read().unwrap();
            engine.on_update(update_dt_measure);
            self.last_update = now;
        }

        if now.duration_since(self.last_render) >= self.render_dt {
            let engine = get_engine();
            let engine = engine.read().unwrap();
            engine.redraw();
            self.last_render = now;
        }

        let next_tick = (self.last_render + self.render_dt)
            .min(self.last_update + self.update_dt);

        // enforces maximum framerate
        event_loop.set_control_flow(
            winit::event_loop::ControlFlow::WaitUntil(next_tick),
        );
    }
}
