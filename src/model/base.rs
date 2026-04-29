use crate::{
    component::Component,
    model::{Material, Mesh},
    Camera, TransformComponent,
};

use super::super::{
    error::*,
    event::{OnEventContext, OnStartContext, OnUpdateContext},
    scene::Scene,
};

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    #[allow(clippy::too_many_arguments)]
    pub fn draw_mesh(
        &mut self,
        mesh_index: usize,
        transform: &TransformComponent,
        camera: &Camera,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Result<()> {
        let mesh = self
            .meshes
            .get_mut(mesh_index)
            .ok_or(Error::Other("Invalid mesh index".to_string()))?;

        // don't render meshes outside of camera view
        if !mesh.is_in_view(&transform.global(), camera) {
            return Ok(());
        }

        let material = self
            .materials
            .get_mut(mesh.material)
            .ok_or(Error::Other("Invalid mesh index".to_string()))?;

        let mesh_buffers = mesh.buffers_ref();
        let mut new_instance_buffer = false;
        let mesh_buffers = match mesh_buffers {
            Some(buffers) => buffers,
            None => {
                // this buffer re-initialisation should be lazy
                mesh.update_buffers(device);
                new_instance_buffer = true;
                mesh.buffers_ref().unwrap()
            }
        };

        if mesh_buffers.empty() {
            return Ok(());
        }

        // update instance buffer (mesh's rendered transform) if it has moved
        if transform.is_dirty() || new_instance_buffer {
            let transform = transform.global();
            let instance_data = [transform.to_raw()];
            let data: &[u8] = bytemuck::cast_slice(&instance_data);
            queue.write_buffer(&mesh_buffers.instance_buffer, 0, data);
        }

        // TODO Do the same thing i did with mesh: optional in buffer makes dirty bit redundant
        if material.dirty {
            material.update_buffers(device, queue, material_layout);
        }
        let material_buffers = material.buffers.as_mut().unwrap();

        render_pass.set_vertex_buffer(0, mesh_buffers.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, mesh_buffers.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            mesh_buffers.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );

        render_pass.set_bind_group(0, &material_buffers.bind_group, &[]);
        render_pass.set_bind_group(1, camera_bind_group, &[]);

        // draw
        let num_elements = mesh.num_indices() as u32;
        render_pass.draw_indexed(0..num_elements, 0, 0..1);

        Ok(())
    }

    pub fn draw_model(
        &mut self,
        transform: &TransformComponent,
        camera: &Camera,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Result<()> {
        for i in 0..self.meshes.len() {
            self.draw_mesh(
                i,
                transform,
                camera,
                device,
                queue,
                render_pass,
                camera_bind_group,
                material_layout,
            )?;
        }
        Ok(())
    }
}

impl Component for Model {
    fn on_start(&mut self, _scene: &mut Scene, _context: OnStartContext) {}

    fn on_update(&mut self, _scene: &mut Scene, _context: OnUpdateContext) {}

    fn on_event(&mut self, _scene: &mut Scene, _context: OnEventContext) {}
}
