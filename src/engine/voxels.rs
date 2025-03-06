use noise::{NoiseFn, Perlin, Seedable};
use wgpu::{util::DeviceExt, Buffer, Device, Queue};

use super::{
    error::*,
    model::{self, Mesh, Model, ModelVertex, Vertex}
};

use crate::engine::resources::load_texture;

const CHUNK_SIZE: usize = 32;
const VOXEL_SIZE: f32 = 1.0;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum VoxelState {
    EMPTY,
    FULL,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoxelChunk {
    // indices: [x][y][z]
    voxels: [[[VoxelState; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl VoxelChunk {
    pub fn new() -> Self {
        let mut chunk = VoxelChunk {
            voxels: [[[VoxelState::EMPTY; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        };

        let perlin = Perlin::new(1);

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let val = perlin.get([
                        (x as f64) / CHUNK_SIZE as f64,
                        (y as f64) / CHUNK_SIZE as f64,
                        (z as f64) / CHUNK_SIZE as f64,
                    ]);
                    println!("{:?}", val);
                    if val > 0.5 {
                        chunk.voxels[x][y][z] = VoxelState::FULL;
                    }
                }
            }     
        }
        chunk
    }

    pub fn get_model(&self, 
        device: &Device, queue: &Queue, 
        layout: &wgpu::BindGroupLayout,
    ) -> Result<Model> {
        let mesh = self.get_mesh(device)?;

        let diffuse_texture = load_texture("cube/cube-diffuse.png", device, queue)?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        });
        
        let material = model::Material {
            name: "Voxel Material".to_string(),
            diffuse_texture,
            bind_group,
        };

        Ok(Model {
            meshes: vec![mesh],
            materials: vec![material],
        })
    }

    fn get_mesh(&self, device: &Device) -> Result<Mesh> {
        let mut vertices = vec![];
        let mut indices = vec![];
        let mut num_verts = 0;
        
        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    if self.voxels[i][j][k] == VoxelState::FULL {
                        let x = (i as f32) * VOXEL_SIZE;
                        let y = (j as f32) * VOXEL_SIZE;
                        let z = (k as f32) * VOXEL_SIZE;

                        if k == CHUNK_SIZE - 1 || self.voxels[i][j][k + 1] == VoxelState::EMPTY {
                            // back face
                            // y
                            // ^
                            // 3 - 2
                            // | / |
                            // 0 - 1 > x
                            vertices.push(ModelVertex {
                                position: [x, y, z + VOXEL_SIZE],
                                tex_coords: [0.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y, z + VOXEL_SIZE],
                                tex_coords: [1.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z + VOXEL_SIZE],
                                tex_coords: [1.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x, y + VOXEL_SIZE, z + VOXEL_SIZE],
                                tex_coords: [0.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            indices.push(num_verts);
                            indices.push(num_verts + 1);
                            indices.push(num_verts + 2);
                            indices.push(num_verts);
                            indices.push(num_verts + 2);
                            indices.push(num_verts + 3);

                            num_verts += 4;
                        }

                        if j == CHUNK_SIZE - 1 || self.voxels[i][j + 1][k] == VoxelState::EMPTY {
                            // top face
                            // 3 - 2 > x
                            // | / |
                            // 0 - 1
                            // v
                            // z
                            vertices.push(ModelVertex {
                                position: [x, y + VOXEL_SIZE, z + VOXEL_SIZE],
                                tex_coords: [0.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z + VOXEL_SIZE],
                                tex_coords: [1.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z],
                                tex_coords: [1.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x, y + VOXEL_SIZE, z],
                                tex_coords: [0.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            indices.push(num_verts);
                            indices.push(num_verts + 1);
                            indices.push(num_verts + 2);
                            indices.push(num_verts);
                            indices.push(num_verts + 2);
                            indices.push(num_verts + 3);
                            
                            num_verts += 4;
                        }

                        if i == CHUNK_SIZE - 1 || self.voxels[i + 1][j][k] == VoxelState::EMPTY {
                            // right face
                            //         y
                            //         ^
                            //     3 - 2
                            //     | / |
                            // z < 0 - 1
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y, z + VOXEL_SIZE],
                                tex_coords: [0.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y, z],
                                tex_coords: [1.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z],
                                tex_coords: [1.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z + VOXEL_SIZE],
                                tex_coords: [0.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            indices.push(num_verts);
                            indices.push(num_verts + 1);
                            indices.push(num_verts + 2);
                            indices.push(num_verts);
                            indices.push(num_verts + 2);
                            indices.push(num_verts + 3);
                            
                            num_verts += 4;
                        }

                        if i == 0 || self.voxels[i - 1][j][k] == VoxelState::EMPTY {
                            // left face
                            // y
                            // ^
                            // 3 - 2
                            // | / |
                            // 0 - 1 > z
                            vertices.push(ModelVertex {
                                position: [x, y, z],
                                tex_coords: [0.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x, y, z + VOXEL_SIZE],
                                tex_coords: [1.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x, y + VOXEL_SIZE, z + VOXEL_SIZE],
                                tex_coords: [1.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x, y + VOXEL_SIZE, z],
                                tex_coords: [0.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            indices.push(num_verts);
                            indices.push(num_verts + 1);
                            indices.push(num_verts + 2);
                            indices.push(num_verts);
                            indices.push(num_verts + 2);
                            indices.push(num_verts + 3);
                            
                            num_verts += 4;
                        }

                        if j == 0 || self.voxels[i][j - 1][k] == VoxelState::EMPTY {
                            // bottom face
                            // z
                            // ^
                            // 3 - 2 > x
                            // | / |
                            // 0 - 1
                            vertices.push(ModelVertex {
                                position: [x, y, z],
                                tex_coords: [0.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y, z],
                                tex_coords: [1.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y, z + VOXEL_SIZE],
                                tex_coords: [1.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x, y, z + VOXEL_SIZE],
                                tex_coords: [0.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            indices.push(num_verts);
                            indices.push(num_verts + 1);
                            indices.push(num_verts + 2);
                            indices.push(num_verts);
                            indices.push(num_verts + 2);
                            indices.push(num_verts + 3);
                            
                            num_verts += 4;
                        }

                        if k == 0 || self.voxels[i][j][k - 1] == VoxelState::EMPTY {
                            // front face
                            //         y
                            //         ^
                            //     3 - 2
                            //     | / |
                            // x < 0 - 1
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y, z],
                                tex_coords: [0.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x, y, z],
                                tex_coords: [1.0, 0.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x, y + VOXEL_SIZE, z],
                                tex_coords: [1.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            vertices.push(ModelVertex {
                                position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z],
                                tex_coords: [0.0, 1.0],
                                normal: [0.0, 0.0, 0.0],
                            });
                            indices.push(num_verts);
                            indices.push(num_verts + 1);
                            indices.push(num_verts + 2);
                            indices.push(num_verts);
                            indices.push(num_verts + 2);
                            indices.push(num_verts + 3);
                            
                            num_verts += 4;
                        }
                    }
                }
            }     
        }

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Some Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor{
                label: Some("Some index Buffer"),
                contents: bytemuck::cast_slice(indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Ok(Mesh {
            name: "Voxel Mesh".to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
        })
    }
}
