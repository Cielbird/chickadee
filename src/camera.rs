use cgmath::{Matrix4, Zero};

use crate::{
    event::{OnEventContext, OnStartContext, OnUpdateContext},
    transform::Transform,
    AxisAlignedBoundingBox, Vector3,
};

use super::{component::Component, scene::Scene};

#[derive(Clone)]
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
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl Component for Camera {
    fn on_start(&mut self, _scene: &mut Scene, _context: OnStartContext) {}

    fn on_update(&mut self, scene: &mut Scene, context: OnUpdateContext) {
        // update projection matrix from entity's transform
        let camera_transform = scene.get_mut_transform(&context.entity);
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
            fovy: 60.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }

    pub fn update_aspect(&mut self, width: f32, height: f32) {
        self.aspect = width / height
    }

    pub fn get_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        self.view_projection_matrix
    }

    // project a world-space point to clip space
    fn to_clip_space(&self, point: Vector3) -> cgmath::Vector4<f32> {
        self.view_projection_matrix * cgmath::Vector4::new(point.x, point.y, point.z, 1.0)
    }

    // does this camera's frustum contain a point
    pub fn contains_point(&self, point: Vector3) -> bool {
        let c = self.to_clip_space(point);
        Self::clip_inside(c)
    }

    // does this camera's frustum contain a bounding box with a given transform
    pub fn contains_bounding_box(
        &self,
        transform: &Transform,
        aabb: &AxisAlignedBoundingBox,
    ) -> bool {
        let corners = [
            aabb.min,
            Vector3::new(aabb.min.x, aabb.min.y, aabb.max.z),
            Vector3::new(aabb.min.x, aabb.max.y, aabb.min.z),
            Vector3::new(aabb.min.x, aabb.max.y, aabb.max.z),
            Vector3::new(aabb.max.x, aabb.min.y, aabb.min.z),
            Vector3::new(aabb.max.x, aabb.min.y, aabb.max.z),
            Vector3::new(aabb.max.x, aabb.max.y, aabb.min.z),
            aabb.max,
        ];

        let clips: Vec<_> = corners
            .iter()
            .map(|&p| self.to_clip_space((*transform) * p))
            .collect();

        if clips.iter().all(|c| c.x < -c.w) {
            return false;
        }
        if clips.iter().all(|c| c.x > c.w) {
            return false;
        }
        if clips.iter().all(|c| c.y < -c.w) {
            return false;
        }
        if clips.iter().all(|c| c.y > c.w) {
            return false;
        }
        if clips.iter().all(|c| c.z < 0.0) {
            return false;
        }
        if clips.iter().all(|c| c.z > c.w) {
            return false;
        }

        true
    }

    // Tests a single clip-space point against all frustum planes
    fn clip_inside(c: cgmath::Vector4<f32>) -> bool {
        if c.w <= 0.0 {
            return false;
        }
        let ndc_x = c.x / c.w;
        let ndc_y = c.y / c.w;
        let ndc_z = c.z / c.w;
        ndc_x >= -1.0
            && ndc_x <= 1.0
            && ndc_y >= -1.0
            && ndc_y <= 1.0
            && ndc_z >= 0.0
            && ndc_z <= 1.0
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
