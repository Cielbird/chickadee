use cgmath::{Matrix4, Transform, Zero};

use crate::event::{OnEventContext, OnStartContext, OnUpdateContext};

use super::{component::Component, scene::Scene};

pub struct Camera {
    view_projection_matrix: cgmath::Matrix4<f32>,

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
    pub(crate) view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
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
    fn on_start(&mut self, _scene: &mut Scene, _context: OnStartContext) {}

    fn on_update(&mut self, scene: &mut Scene, context: OnUpdateContext) {
        // update projection matrix from entity's transform
        let camera_transform = scene.get_transform(&context.entity);
        let camera_transform = camera_transform.read().unwrap();
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        self.view_projection_matrix =
            OPENGL_TO_WGPU_MATRIX * proj * camera_transform.global().inverse().as_matrix();
    }

    fn on_event(&mut self, _scene: &mut Scene, _context: OnEventContext) {}
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            view_projection_matrix: Matrix4::zero(),
            aspect: 1.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn update_aspect(&mut self, width: f32, height: f32) {
        self.aspect = width / height
    }

    pub fn get_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        self.view_projection_matrix
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
