use std::sync::{Arc, RwLock, Weak};

use cgmath::{Matrix4, One, Point3, Quaternion, Transform, Vector3, Zero};
use winit::event::WindowEvent;

use crate::engine::transform;

use super::{component::Component, entity::Entity, scene::Scene};

pub struct Camera {
    entity: Option<Weak<RwLock<Entity>>>,

    pub transform: transform::Transform,

    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, scene: &Scene) {
        if let Some(cam) = scene.find_first_component::<Camera>() {
            if let Ok(cam) = cam.get_ref() {
                self.view_proj = cam.get_view_projection_matrix().into();
            }
        } else {
            println!("No camera in scene!");
        }
    }
}

// necessary because cgmath uses opengl style coords, and wgpu uses directx style coords
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl Component for Camera {
    fn get_entity(&self) -> Option<Arc<RwLock<Entity>>> {
        if let Some(e) = &self.entity {
            return Some(e.upgrade().unwrap());
        }
        return None;
    }

    fn set_entity(&mut self, entity: &Arc<RwLock<Entity>>) {
        self.entity = Some(Arc::downgrade(entity));
    }

    fn on_start(&mut self, scene: &mut Scene) {
        return;
    }

    fn on_update(&mut self, scene: &mut Scene) {
        return;
    }

    fn on_event(&mut self, scene: &mut Scene, event: &WindowEvent) {
        return;
    }
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            entity: None,
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            transform: transform::Transform{
                position: Point3 {
                    x: 0.0, y: 1.0, z: 2.0,
                },
                rotation: Quaternion::one(),
                scale: Vector3{ x: 1.0, y: 1.0, z: 1.0 },
            },
            aspect: 1., //config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    fn get_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        if let Some(entity) = self.get_entity() {
            if let Ok(entity) = entity.read() {
                let transform = entity.get_tranform();
                return OPENGL_TO_WGPU_MATRIX
                    * proj
                    * transform.matrix().inverse_transform().unwrap();
            }
        }

        Matrix4::zero()
    }
}
