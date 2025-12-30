use std::sync::{Arc, Mutex, RwLock};

use winit::{event::WindowEvent, window::Window};

use crate::engine::handler::EngineHandler;

use super::{render::Renderer, scene::Scene};

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
    let instance = instance.clone().unwrap();
    return instance;
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

    pub fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        self.window = Some(window.clone());

        let renderer = Renderer::new(window, self.scene.clone());
        self.renderer = Some(Mutex::new(renderer))
    }

    pub fn get_window(&self) -> Arc<Window> {
        self.window.clone().unwrap()
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

    pub fn on_update(&self) {
        let scene_ref = &mut self.scene.write().unwrap();
        scene_ref.on_update();
    }

    pub fn on_event(&self, event: &WindowEvent) {
        let scene_ref = &mut self.scene.write().unwrap();
        scene_ref.on_event(event);
    }
}
