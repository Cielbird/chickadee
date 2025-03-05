mod engine;
mod app;
mod camera_controller;

use app::Application;
use winit::event_loop::EventLoop;

pub async fn run() {

    let event_loop = EventLoop::new().unwrap();

    let mut window_state = Application::new();
    let _ = event_loop.run_app(&mut window_state);
}

fn main() {
    // blocking call
    pollster::block_on(run());
}
