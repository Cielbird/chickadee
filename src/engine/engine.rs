use std::sync::{Arc, RwLock};

use winit::{event::WindowEvent, window::Window};

use crate::camera_controller::CameraController;

use super::{camera::Camera, renderer::Renderer, scene::Scene};

pub struct Engine<'a> {
    pub renderer: Renderer<'a>,
    scene: Arc<RwLock<Scene>>,
}

impl<'a> Engine<'a> {
    pub fn new(window: Arc<Window>) -> Self {
        let mut scene = Scene::new();

        let root = scene.get_root();

        // let voxels = Entity::add_child(&root);
        // Entity::add_component(&voxels, component);

        let player = scene
            .add_entity(root.clone(), "player".to_string())
            .unwrap();

        let player_cam = scene
            .add_entity(player.clone(), "player_cam".to_string())
            .unwrap();

        let camera = Camera::new();
        scene.add_component(player_cam.clone(), camera).unwrap();

        let mut camera_ctrl = CameraController::new(1.);
        camera_ctrl.window = Some(window.clone());
        scene.add_component(player_cam.clone(), camera_ctrl).unwrap();

        let scene = Arc::new(RwLock::new(scene));
        let renderer = Renderer::new(window, scene.clone());
        Self { renderer, scene }
    }

    pub fn on_start(&mut self) {
        let scene_ref = &mut self.scene.write().unwrap();
        scene_ref.on_start();
    }

    pub fn on_update(&mut self) {
        let scene_ref = &mut self.scene.write().unwrap();
        scene_ref.on_update();
    }

    pub fn on_event(&mut self, event: &WindowEvent) {
        let scene_ref = &mut self.scene.write().unwrap();
        scene_ref.on_event(event);
    }
}
