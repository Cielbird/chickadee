mod camera_controller;
mod engine;

use crate::{
    camera_controller::CameraController,
    engine::{camera::Camera, engine::Engine, scene::Scene},
};

fn main() {
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

    let camera_ctrl = CameraController::new();
    scene
        .add_component(player_cam.clone(), camera_ctrl)
        .unwrap();

    Engine::run(scene);
}
