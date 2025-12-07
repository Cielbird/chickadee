use cgmath::Point3;
use noise::{NoiseFn, Perlin};
use wgpu::{util::DeviceExt, Device, Queue};

use super::{
    error::*,
    model::{self, Mesh, Model, ModelVertex},
};

use crate::engine::resources::load_texture;

const CHUNK_SIZE: usize = 8;
const VOXEL_SIZE: f32 = 1.0;

const TEX_SIZE: usize = 128;
const TEX_FACE_SIZE: usize = 8;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum VoxelState {
    EMPTY,
    FULL,
}

#[derive(Debug)]
enum FaceDir {
    X_POS,
    X_NEG,
    Y_POS,
    Y_NEG,
    Z_POS,
    Z_NEG,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VoxelChunk {
    // indices: [x][y][z]
    voxels: [[[VoxelState; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    chunk_pos: cgmath::Point3<i32>,
}

impl VoxelChunk {
    pub fn new() -> Self {
        let mut chunk = VoxelChunk {
            voxels: [[[VoxelState::EMPTY; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            chunk_pos: Point3::<i32>::new(0, 0, 0),
        };

        let perlin = Perlin::new(20);

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let val = perlin.get([
                        (x as f64) / CHUNK_SIZE as f64,
                        (y as f64) / CHUNK_SIZE as f64,
                        (z as f64) / CHUNK_SIZE as f64,
                    ]) / ((y as f64) / CHUNK_SIZE as f64 * 2.);
                    if val > 0.5 {
                        chunk.voxels[x][y][z] = VoxelState::FULL;
                    }
                }
            }
        }
        chunk
    }

    pub fn get_model(
        &self,
        device: &Device,
        queue: &Queue,
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
        let mut indices: Vec<i32> = vec![];
        let mut num_verts = 0;

        for i in 0..CHUNK_SIZE {
            for j in 0..CHUNK_SIZE {
                for k in 0..CHUNK_SIZE {
                    if self.voxels[i][j][k] == VoxelState::FULL {
                        self.add_voxel_faces(
                            &mut vertices,
                            &mut indices,
                            &mut num_verts,
                            Point3 { x: i, y: j, z: k },
                        );
                    }
                }
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Some Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Some index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let mesh = Mesh {
            name: "Voxel Mesh".to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
        };
        println!("mesh: {:?}", mesh);

        Ok(mesh)
    }

    fn add_voxel_faces(
        &self,
        vertices: &mut Vec<ModelVertex>,
        indices: &mut Vec<i32>,
        num_verts: &mut usize,
        voxel_indices: cgmath::Point3<usize>,
    ) {
        // indices in chunck
        let i = voxel_indices.x;
        let j = voxel_indices.y;
        let k = voxel_indices.z;

        // global positions
        let chunk_origin = self.chunk_pos * CHUNK_SIZE as i32;
        let global_pos = Point3 {
            x: chunk_origin.x as f32 + voxel_indices.x as f32 * VOXEL_SIZE,
            y: chunk_origin.x as f32 + voxel_indices.y as f32 * VOXEL_SIZE,
            z: chunk_origin.x as f32 + voxel_indices.z as f32 * VOXEL_SIZE,
        };

        if k < CHUNK_SIZE - 1 && self.voxels[i][j][k + 1] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::Z_POS, global_pos);
        }

        if j < CHUNK_SIZE - 1 && self.voxels[i][j + 1][k] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::Y_POS, global_pos);
        }

        if i < CHUNK_SIZE - 1 && self.voxels[i + 1][j][k] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::X_POS, global_pos);
        }

        if i > 0 && self.voxels[i - 1][j][k] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::X_NEG, global_pos);
        }

        if j > 0 && self.voxels[i][j - 1][k] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::Y_NEG, global_pos);
        }

        if k > 0 && self.voxels[i][j][k - 1] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::Z_NEG, global_pos);
        }
    }

    fn add_voxel_face(
        vertices: &mut Vec<ModelVertex>,
        indices: &mut Vec<i32>,
        num_verts: &mut usize,
        facing_dir: FaceDir,
        origin: Point3<f32>,
    ) {
        let d_uv = (TEX_FACE_SIZE as f32) / (TEX_SIZE as f32);
        let x = origin.x;
        let y = origin.y;
        let z = origin.z;
        match facing_dir {
            FaceDir::Z_POS => {
                // back face
                // y
                // ^
                // 3 - 2
                // | / |
                // 0 - 1 > x
                let (u, v) = (2., 0.);
                vertices.push(ModelVertex {
                    position: [x, y, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::Y_POS => {
                // top face
                // 3 - 2 > x
                // | / |
                // 0 - 1
                // v
                // z
                let (u, v) = (3., 1.);
                vertices.push(ModelVertex {
                    position: [x, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x, y + VOXEL_SIZE, z],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::X_POS => {
                // right face
                //         y
                //         ^
                //     3 - 2
                //     | / |
                // z < 0 - 1
                let (u, v) = (2., 0.);
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y, z],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::X_NEG => {
                // left face
                // y
                // ^
                // 3 - 2
                // | / |
                // 0 - 1 > z
                let (u, v) = (2., 0.);
                vertices.push(ModelVertex {
                    position: [x, y, z],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x, y, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x, y + VOXEL_SIZE, z],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::Y_NEG => {
                // bottom face
                // z
                // ^
                // 3 - 2 > x
                // | / |
                // 0 - 1
                let (u, v) = (2., 1.);
                vertices.push(ModelVertex {
                    position: [x, y, z],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y, z],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x, y, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::Z_NEG => {
                // front face
                //         y
                //         ^
                //     3 - 2
                //     | / |
                // x < 0 - 1
                let (u, v) = (2., 0.);
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y, z],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x, y, z],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x, y + VOXEL_SIZE, z],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(ModelVertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            _ => return,
        }

        let first_index = *num_verts as i32;
        indices.push(first_index);
        indices.push(first_index + 1);
        indices.push(first_index + 2);
        indices.push(first_index);
        indices.push(first_index + 2);
        indices.push(first_index + 3);
        *num_verts += 4;
    }
}
