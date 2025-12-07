use std::{
    sync::Weak,
    sync::{Arc, RwLock},
};

use cgmath::{Vector3, Zero};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::engine::{component::Component, entity::Entity, scene::Scene};

use super::engine::camera::Camera;

pub struct CameraController {
    entity: Option<Weak<RwLock<Entity>>>,

    walk_speed: f32,
    sprint_speed: f32,
    cam_speed: f32,
    pub window: Option<Arc<Window>>,
    mouse_captured: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_sprint_pressed: bool,
    delta_yaw: f32,
    delta_pitch: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            entity: None,

            walk_speed: 0.01,
            sprint_speed: 0.05,
            cam_speed: 0.001,
            window: None,
            mouse_captured: true,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            is_sprint_pressed: false,
            delta_yaw: 0.,
            delta_pitch: 0.,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        let mut move_vec = Vector3::<f32>::zero();
        let mut global_move_vec = Vector3::<f32>::zero();

        // needs entity hierarchy for correct movement
        if self.is_forward_pressed {
            move_vec += Vector3::new(0., 0., -1.);
        }
        if self.is_backward_pressed {
            move_vec += Vector3::new(0., 0., 1.);
        }

        if self.is_right_pressed {
            move_vec += Vector3::new(1., 0., 0.);
        }
        if self.is_left_pressed {
            move_vec += Vector3::new(-1., 0., 0.);
        }

        if self.is_up_pressed {
            global_move_vec += Vector3::new(0., 1., 0.);
        }
        if self.is_down_pressed {
            global_move_vec += Vector3::new(0., -1., 0.);
        }

        if self.is_sprint_pressed {
            move_vec *= self.sprint_speed;
            global_move_vec *= self.sprint_speed;
        } else {
            move_vec *= self.walk_speed;
            global_move_vec *= self.walk_speed;
        }

        //camera.transform.move_local(move_vec);
        //camera.transform.move_global(global_move_vec);

        // TODO this should be solved with a transform hierachy

        // local rotation for the pitch
        let rotation = Vector3 {
            x: self.delta_pitch * self.cam_speed,
            y: 0.,
            z: 0.,
        };
        //camera.transform.rotate_euler_local(rotation);
        // global rotation for the yaw
        let rotation = Vector3 {
            x: 0.,
            y: self.delta_yaw * self.cam_speed,
            z: 0.,
        };
        //camera.transform.rotate_euler_global(rotation);

        // reset rotation deltas
        self.delta_yaw = 0.;
        self.delta_pitch = 0.;
    }

    fn toggle_mouse_captured(&mut self) {
        self.set_mouse_captured(!self.mouse_captured);
    }

    pub fn set_mouse_captured(&mut self, captured: bool) {
        self.mouse_captured = captured;
        let mode = if self.mouse_captured {
            winit::window::CursorGrabMode::Locked
        } else {
            winit::window::CursorGrabMode::None
        };
        self.window.as_mut().unwrap().set_cursor_grab(mode).ok();
        self.window
            .as_mut()
            .unwrap()
            .set_cursor_visible(!self.mouse_captured);
    }
}

impl Component for CameraController {
    fn get_entity(&self) -> Option<Arc<RwLock<Entity>>> {
        if let Some(e) = &self.entity {
            return Some(e.upgrade().unwrap());
        }
        return None;
    }

    fn set_entity(&mut self, entity: &Arc<RwLock<Entity>>) {
        self.entity = Some(Arc::downgrade(entity));
    }

    fn on_start(&mut self, scene: &mut Scene) {}

    fn on_update(&mut self, scene: &mut Scene) {
        let cam_ptr = scene.find_first_component::<Camera>();
        if let Some(cam_ptr) = cam_ptr {
            self.update_camera(&mut cam_ptr.write().unwrap())
        }
    }

    fn on_event(&mut self, scene: &mut Scene, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                    }
                    KeyCode::KeyE | KeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                    }
                    KeyCode::KeyQ | KeyCode::ControlLeft => {
                        self.is_down_pressed = is_pressed;
                    }
                    KeyCode::ShiftLeft => {
                        self.is_sprint_pressed = is_pressed;
                    }
                    KeyCode::Escape => {
                        if *state == ElementState::Pressed {
                            self.toggle_mouse_captured();
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } if self.mouse_captured => {
                let (x, y) = (position.x as f32, position.y as f32);
                let window = self.window.as_mut().unwrap();
                let center_x = window.inner_size().width as f32 / 2.;
                let center_y = window.inner_size().height as f32 / 2.;

                self.delta_yaw = x - center_x;
                self.delta_pitch = y - center_y;

                // Reset cursor to center
                window
                    .set_cursor_position(PhysicalPosition::new(center_x, center_y))
                    .ok();
            }
            _ => {}
        }
    }
}
