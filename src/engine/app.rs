use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::camera_controller::CameraController;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use super::renderer::Renderer;
use super::scene::Scene;

pub type StartCallbackType = Box<dyn Fn(Arc<RwLock<Scene>>)>;
pub type UpdateCallbackType = Box<dyn Fn(Arc<RwLock<Scene>>)>;
pub type EventCallbackType = Box<dyn Fn(Arc<RwLock<Scene>>, WindowEvent)>;

pub struct Application<'a> {
    renderer: Option<Renderer<'a>>,
    scene: Option<Arc<RwLock<Scene>>>,

    on_start: StartCallbackType,
    on_update: UpdateCallbackType,
    on_event: EventCallbackType,
}

impl<'a> Application<'a> {
    pub fn new(on_start: StartCallbackType, on_update: UpdateCallbackType, on_event: EventCallbackType) -> Self {
        Self {
            renderer: None,
            scene: None,
            on_start,
            on_update,
            on_event,
        }
    }
}

impl<'a> ApplicationHandler for Application<'a>{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_attribs = Window::default_attributes()
            .with_title("Hello!");
        let window = event_loop.create_window(win_attribs).unwrap();
        let window_ptr = Arc::new(window);
        let scene = Arc::new(RwLock::new(Scene::new()));

        (self.on_start)(Arc::clone(&scene));

        let renderer = Renderer::new(Arc::clone(&window_ptr), Arc::clone(&scene));
        self.renderer = Some(renderer);
        self.scene = Some(scene);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if self.renderer.is_none() || self.scene.is_none() {
            return;
        }
        let renderer = self.renderer.as_mut().unwrap();
        let scene = self.scene.as_mut().unwrap();
        let window = renderer.window();

        let target_frame_time = Duration::from_secs_f64(1.0 / 60.0); // 60 fps
        let mut last_frame_time = Instant::now();

        if window.id() == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_frame_time);
    
                    if elapsed < target_frame_time {
                        let sleep_time = target_frame_time - elapsed;
                        println!("FPS: {}", 1.0/elapsed.as_secs_f32());
                        std::thread::sleep(sleep_time);
                    }
    
                    last_frame_time = Instant::now();

                    renderer.render().unwrap();
                    
                    (self.on_update)(Arc::clone(scene));
                }
                _ => { }
            }

            (self.on_event)(Arc::clone(scene), event);
        }
    }

    // TODO see if i can delete ts
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let renderer = self.renderer.as_mut().unwrap();
        let window = renderer.window();
        window.request_redraw();
        println!("woah don't delete me")
    }
}
