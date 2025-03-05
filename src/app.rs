use crate::camera_controller::CameraController;
use crate::engine::engine::Engine;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct Application<'a> {
    engine: Option<Engine<'a>>,
    camera_controller: CameraController,
}

impl<'a> Application<'a> {
    pub fn new() -> Self {
        let camera_controller = CameraController::new(0.02);
        Self {
            engine: None,
            camera_controller: camera_controller,
        }
    }

    pub fn on_event(&mut self, event: &WindowEvent) {
        self.camera_controller.process_events(&event);
    }

    pub fn on_update(&mut self) {
        let renderer = &mut self.engine.as_mut().unwrap().renderer;
        self.camera_controller.update_camera(&mut renderer.camera);
    }
}

impl<'a> ApplicationHandler for Application<'a>{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes().with_title("Hello!")).unwrap();
        self.engine = Some(Engine::new(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if self.engine.is_none() {
            return;
        }
        let renderer = &mut self.engine.as_mut().unwrap().renderer;
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
                    
                    self.on_update();
                }
                _ => {
                    self.on_event(&event);
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let renderer = &mut self.engine.as_mut().unwrap().renderer;
        let window = renderer.window();
        window.request_redraw();
    }
}
