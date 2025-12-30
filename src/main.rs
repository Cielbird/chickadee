mod camera_controller;
mod engine;

use cgmath::Vector3;

use crate::{
    camera_controller::CameraController,
    engine::{camera::Camera, engine::Engine, resources::load_model, scene::Scene},
};

fn main() {
    let mut scene = Scene::new();
    let root = scene.get_root();

    let player = scene
        .add_entity(root.clone(), "player".to_string())
        .unwrap();

    let player_cam = scene
        .add_entity(player.clone(), "player_cam".to_string())
        .unwrap();

    let cam_transform = scene.get_transform_mut(&player_cam).unwrap();
    cam_transform.move_global(Vector3::new(0., 2., 0.));

    let camera = Camera::new();
    scene.add_component(player_cam.clone(), camera).unwrap();

    let camera_ctrl = CameraController::new();
    scene
        .add_component(player_cam.clone(), camera_ctrl)
        .unwrap();

    let block_entity = scene.add_entity(root, "block".to_string()).unwrap();
    let model_fut = load_model("cube/cube.obj");
    let model = pollster::block_on(model_fut).expect("coulnd't load model");

    scene.add_component(block_entity, model).unwrap();

    let model_fut = load_model("cube/cube.obj");
    let model = pollster::block_on(model_fut).expect("coulnd't load model");
    scene.add_component(player, model).unwrap();

    Engine::run(scene);
}
