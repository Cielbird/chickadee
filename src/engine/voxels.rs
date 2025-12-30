use cgmath::Point3;
use noise::{NoiseFn, Perlin};

use crate::engine::resources::load_image;

use super::{
    error::*,
    model::{self, Mesh, Model, Vertex},
};

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

#[allow(unused)]
#[derive(Debug)]
enum FaceDir {
    XPos,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
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

    pub fn get_model(&self) -> Result<Model> {
        let mesh = self.get_mesh()?;

        let diffuse_image = load_image("cube/cube-diffuse.png")?;

        let material = model::Material {
            name: "Voxel Material".to_string(),
            diffuse_image,
            dirty: true,
            buffers: None,
        };

        Ok(Model {
            meshes: vec![mesh],
            materials: vec![material],
        })
    }

    fn get_mesh(&self) -> Result<Mesh> {
        let mut vertices = vec![];
        let mut indices: Vec<u32> = vec![];
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

        let mesh = Mesh {
            // TODO new constructor for this
            name: "Voxel Mesh".to_string(),
            vertices,
            indices,
            material: 0,
            dirty: true,
            buffers: None,
        };

        Ok(mesh)
    }

    fn add_voxel_faces(
        &self,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
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
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::ZPos, global_pos);
        }

        if j < CHUNK_SIZE - 1 && self.voxels[i][j + 1][k] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::YPos, global_pos);
        }

        if i < CHUNK_SIZE - 1 && self.voxels[i + 1][j][k] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::XPos, global_pos);
        }

        if i > 0 && self.voxels[i - 1][j][k] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::XNeg, global_pos);
        }

        if j > 0 && self.voxels[i][j - 1][k] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::YNeg, global_pos);
        }

        if k > 0 && self.voxels[i][j][k - 1] == VoxelState::EMPTY {
            Self::add_voxel_face(vertices, indices, num_verts, FaceDir::ZNeg, global_pos);
        }
    }

    fn add_voxel_face(
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        num_verts: &mut usize,
        facing_dir: FaceDir,
        origin: Point3<f32>,
    ) {
        let d_uv = (TEX_FACE_SIZE as f32) / (TEX_SIZE as f32);
        let x = origin.x;
        let y = origin.y;
        let z = origin.z;
        match facing_dir {
            FaceDir::ZPos => {
                // back face
                // y
                // ^
                // 3 - 2
                // | / |
                // 0 - 1 > x
                let (u, v) = (2., 0.);
                vertices.push(Vertex {
                    position: [x, y, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::YPos => {
                // top face
                // 3 - 2 > x
                // | / |
                // 0 - 1
                // v
                // z
                let (u, v) = (3., 1.);
                vertices.push(Vertex {
                    position: [x, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x, y + VOXEL_SIZE, z],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::XPos => {
                // right face
                //         y
                //         ^
                //     3 - 2
                //     | / |
                // z < 0 - 1
                let (u, v) = (2., 0.);
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y, z],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::XNeg => {
                // left face
                // y
                // ^
                // 3 - 2
                // | / |
                // 0 - 1 > z
                let (u, v) = (2., 0.);
                vertices.push(Vertex {
                    position: [x, y, z],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x, y, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x, y + VOXEL_SIZE, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x, y + VOXEL_SIZE, z],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::YNeg => {
                // bottom face
                // z
                // ^
                // 3 - 2 > x
                // | / |
                // 0 - 1
                let (u, v) = (2., 1.);
                vertices.push(Vertex {
                    position: [x, y, z],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y, z],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y, z + VOXEL_SIZE],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x, y, z + VOXEL_SIZE],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
            }
            FaceDir::ZNeg => {
                // front face
                //         y
                //         ^
                //     3 - 2
                //     | / |
                // x < 0 - 1
                let (u, v) = (2., 0.);
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y, z],
                    uvs: [d_uv * u, d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x, y, z],
                    uvs: [d_uv * (u + 1.), d_uv * (v + 1.)],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x, y + VOXEL_SIZE, z],
                    uvs: [d_uv * (u + 1.), d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
                vertices.push(Vertex {
                    position: [x + VOXEL_SIZE, y + VOXEL_SIZE, z],
                    uvs: [d_uv * u, d_uv * v],
                    normal: [0.0, 0.0, 0.0],
                });
            }
        }

        let first_index = *num_verts as u32;
        indices.push(first_index);
        indices.push(first_index + 1);
        indices.push(first_index + 2);
        indices.push(first_index);
        indices.push(first_index + 2);
        indices.push(first_index + 3);
        *num_verts += 4;
    }
}
