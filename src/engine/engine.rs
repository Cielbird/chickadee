use std::{path::Component, sync::{Arc, RwLock}};

use winit::{event::WindowEvent, window::Window};

use super::{camera::Camera, entity::{Entity}, renderer::Renderer, scene::Scene, transform::Transform};

pub struct Engine<'a> {
    pub renderer: Renderer<'a>,
    scene: Arc<RwLock<Scene>>,
}

impl<'a> Engine<'a> {
    pub fn new(window: Arc<Window>) -> Self {
        let mut scene = Scene::new();

        let root = scene.get_root();

        let player = Entity::add_child(&root);

        let player_cam = Entity::add_child(&player);
        let camera = Camera::new();
        Entity::add_component::<Camera>(&player_cam, camera);

        let scene = Arc::new(RwLock::new(scene));
        let renderer = Renderer::new(window, scene.clone());
        Self {
            renderer,
            scene,
        }
    }

    pub fn on_start(&mut self) {
        let scene_ref = &mut self.scene.write().unwrap();
        for entity in scene_ref.get_entity_iter() {
            entity.write().unwrap().on_start(scene_ref);
        }
    }

    pub fn on_update(&mut self) {
        let scene_ref = &mut self.scene.write().unwrap();
        for entity in scene_ref.get_entity_iter() {
            entity.write().unwrap().on_update(scene_ref);
        }
    }

    pub fn on_event(&mut self, event: &WindowEvent) {
        let scene_ref = &mut self.scene.write().unwrap();
        for entity in scene_ref.get_entity_iter() {
            entity.write().unwrap().on_event(scene_ref, event);
        }
    }
}
