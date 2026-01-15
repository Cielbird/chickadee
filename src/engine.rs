use std::{sync::{Arc, Mutex, RwLock}, time::Duration};

use winit::{dpi::PhysicalPosition, event::WindowEvent, window::Window};

use crate::handler::EngineHandler;

use super::{render::Renderer, scene::Scene};

use crate::error::*;

use winit::event_loop::EventLoop;

pub struct Engine {
    pub(crate) renderer: Option<Mutex<Renderer<'static>>>,
    window: Option<Arc<Window>>,
    scene: Arc<RwLock<Scene>>,
}

static ENGINE_INSTANCE: Mutex<Option<Arc<RwLock<Engine>>>> = Mutex::new(None);

pub fn get_engine() -> Arc<RwLock<Engine>> {
    let mut instance = ENGINE_INSTANCE.lock().unwrap();
    if instance.is_none() {
        let engine = Arc::new(RwLock::new(Engine::new()));
        *instance = Some(engine);
    }

    instance.clone().unwrap()
}

impl Engine {
    fn new() -> Self {
        Self {
            renderer: None,
            window: None,
            scene: Arc::new(RwLock::new(Scene::new())),
        }
    }

    pub fn run(scene: Scene) {
        Self::set_scene(scene);
        pollster::block_on(async move {
            let event_loop = EventLoop::new().unwrap();

            let mut window_state = EngineHandler::new();

            let _ = event_loop.run_app(&mut window_state);
        });
    }

    pub(crate) fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        self.window = Some(window.clone());

        let renderer = Renderer::new(window, self.scene.clone());
        self.renderer = Some(Mutex::new(renderer))
    }

    /// Set the capture state of the cursor. When captured, the cursor should be locked and
    /// invisible, otherwise, it is free and visible
    pub fn set_cursor_captured(&self, captured: bool) {
        if let Some(window) = &self.window {
            let mode = if captured {
                winit::window::CursorGrabMode::Locked
            } else {
                winit::window::CursorGrabMode::None
            };
            window.set_cursor_grab(mode).ok();
            window.set_cursor_visible(!captured);
        }
    }

    pub fn set_cursor_position(&self, x: f32, y: f32) -> Result<()> {
        if let Some(window) = &self.window {
            window
                .set_cursor_position(PhysicalPosition::new(x, y))
                .map_err(|e| Error::Other(e.to_string()))?;
            Ok(())
        } else {
            Err(Error::Other("Nonexistant window".to_string()))
        }
    }

    /// Returns (width, height)
    pub fn window_size(&self) -> Option<(u32, u32)> {
        if let Some(window) = &self.window {
            let size = window.inner_size();
            Some((size.width, size.height))
        } else {
            None
        }
    }

    pub fn redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub fn set_scene(scene: Scene) {
        let engine = get_engine();
        let engine = engine.write().unwrap();
        let mut s = engine.scene.write().unwrap();
        *s = scene;
    }

    pub fn on_start(&self) {
        let scene_ref = &mut self.scene.write().unwrap();
        scene_ref.on_start();
    }

    pub fn on_update(&self, delta_time: Duration) {
        let scene_ref = &mut self.scene.write().unwrap();
        scene_ref.on_update(delta_time);
    }

    pub fn on_event(&self, event: &WindowEvent) {
        let scene_ref = &mut self.scene.write().unwrap();
        scene_ref.on_event(event);
    }
}
