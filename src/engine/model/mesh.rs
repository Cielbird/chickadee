use wgpu::{util::DeviceExt as _, Device};

use crate::engine::model::ModelVertex;

#[derive(Debug)]
pub struct Mesh {
    pub name: String,

    // CPU buffer
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub material: usize,

    pub dirty: bool, // true if CPU buffers have been changed but not GPU buffers
    pub buffers: Option<MeshBuffers>,
}

#[derive(Debug)]
pub struct MeshBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Mesh {
    /// Update GPU buffers according to data in CPU buffers
    pub fn update_buffers(&mut self, device: &Device) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Some Vertex Buffer"),
            contents: bytemuck::cast_slice(self.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Some index Buffer"),
            contents: bytemuck::cast_slice(self.indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        self.buffers = Some(MeshBuffers {
            vertex_buffer,
            index_buffer,
        });
        self.dirty = false;
    }
}
