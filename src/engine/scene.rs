use super::{
    error::*,
    model::Model
};

pub struct Scene {
    pub models: Vec<Model>,
}

impl Scene {
    pub fn draw_scene(
        &self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup
    ) -> Result<()> {
        for i in 0..self.models.len() {
            self.models.get(i).unwrap().draw_model(render_pass, camera_bind_group)?;
        }
        Ok(())
    }
}
