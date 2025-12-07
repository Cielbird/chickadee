mod app;
mod camera_controller;
mod engine;

use app::Application;
use winit::event_loop::EventLoop;

pub async fn run() {
    // TODO change how code is written with this gameengine
    let event_loop = EventLoop::new().unwrap();

    let mut window_state = Application::new();

    let _ = event_loop.run_app(&mut window_state);
}

fn main() {
    // blocking call
    pollster::block_on(run());
}
