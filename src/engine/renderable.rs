use crate::engine::component::Component;
use super::error::*;

/// Allows a component to be rendered
pub trait RenderableComponent: Component {
    fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
    ) -> Result<()>;
}
