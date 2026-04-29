use cgmath::Vector3;
use wgpu::{util::DeviceExt as _, Device};

use crate::{model::Vertex, transform::Transform, AxisAlignedBoundingBox, Camera};

#[derive(Debug)]
pub struct Mesh {
    #[allow(unused)]
    pub name: String,

    // CPU buffer
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    pub material: usize, // buffers don't depend on this

    buffers: Option<MeshBuffers>,
    bounding_box: Option<AxisAlignedBoundingBox>,
}

#[derive(Debug)]
pub struct MeshBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
}

/// Used for representing each instance (it's transform) in the shader
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
    model: [[f32; 4]; 4],
}

impl Mesh {
    pub fn new(name: String, vertices: Vec<Vertex>, indices: Vec<u32>, material: usize) -> Self {
        Self {
            name,
            vertices,
            indices,
            material,
            buffers: None,
            bounding_box: None,
        }
    }

    pub fn set_vertices(&mut self, vertices: Vec<Vertex>) {
        self.vertices = vertices;
        // deletes buffers
        self.buffers = None;
        self.bounding_box = None;
    }

    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.indices = indices;
        // deletes buffers
        self.buffers = None;
        self.bounding_box = None;
    }

    pub fn buffers_ref(&mut self, device: &Device) -> &MeshBuffers {
        if self.buffers.is_some() {
            return self.buffers.as_ref().unwrap();
        }

        // need to update GPU buffers
        self.update_buffers(device);
        return self.buffers.as_ref().unwrap();
    }

    pub fn aabb_ref(&mut self) -> &AxisAlignedBoundingBox {
        if self.bounding_box.is_some() {
            return self.bounding_box.as_ref().unwrap();
        }

        // need to update bounding box
        self.update_bounding_box();
        return self.bounding_box.as_ref().unwrap();
    }

    pub fn num_indices(&self) -> usize {
        self.indices.len()
    }

    // update GPU buffers without touching self.dirty
    fn update_buffers(&mut self, device: &Device) {
        // destroy current buffers
        self.buffers = None;

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

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[Transform::identity().to_raw()]), // only one instance
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        self.buffers = Some(MeshBuffers {
            vertex_buffer,
            index_buffer,
            instance_buffer,
        });
    }

    // update the bounding box according to CPU buffers
    fn update_bounding_box(&mut self) {
        // update bounding box
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;

        for v in &self.vertices {
            let [x, y, z] = v.position;
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
            if z < min_z {
                min_z = z;
            }
            if z > max_z {
                max_z = z;
            }
        }

        let position = Vector3::new(
            (min_x + max_x) / 2.,
            (min_y + max_y) / 2.,
            (min_z + max_z) / 2.,
        );
        let dimensions = Vector3::new(max_x - min_x, max_y - min_y, max_z - min_z);
        self.bounding_box = Some(AxisAlignedBoundingBox::new(position, dimensions));
    }

    pub fn is_in_view(&mut self, transform: &Transform, camera: &Camera) -> bool {
        let aabb = self.aabb_ref();
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

        corners
            .iter()
            .any(|point| camera.contains_point((*transform) * (*point)))
    }
}

impl MeshBuffers {
    pub fn empty(&self) -> bool {
        self.vertex_buffer.size() == 0
    }
}

impl TransformRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TransformRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Transform {
    pub fn to_raw(self) -> TransformRaw {
        TransformRaw {
            model: self.as_matrix().into(),
        }
    }
}
