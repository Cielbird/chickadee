use std::{
    io::{BufReader, Cursor},
    path::Path,
};

use crate::engine::transform::Transform;

use super::{error::*, model};

fn load_binary(file_name: &str) -> Result<Vec<u8>> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let buffer = std::fs::read(path)?;
    Ok(buffer)
}

pub fn load_string(file_name: &str) -> Result<String> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let txt = std::fs::read_to_string(path)?;
    Ok(txt)
}

pub fn load_image(file_name: &str) -> Result<image::DynamicImage> {
    let data = load_binary(file_name)?;
    image::load_from_memory(&data).map_err(|e| Error::ImageError(e))
}

pub async fn load_model(file_name: &str) -> Result<model::Model> {
    // path all files for the model will be relative to
    let parent_path = Path::new(file_name)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    let obj_text = load_string(file_name)?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let pb = parent_path.join(p);
            let p = pb.to_str().expect("fatal path error");
            let mat_text = load_string(&p).unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        // texure path is relative to material
        let pb = parent_path.join(m.diffuse_texture);
        let p = pb.to_str().expect("fatal path error");
        let diffuse_texture = load_image(p)?;

        materials.push(model::Material {
            name: m.name,
            diffuse_image: diffuse_texture,
            dirty: true,
            buffers: None,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            uvs: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                            normal: [0.0, 0.0, 0.0],
                        }
                    } else {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            uvs: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let indices = m.mesh.indices.clone();

            let material = m.mesh.material_id.unwrap_or(0);

            model::Mesh {
                name: file_name.to_string(),
                vertices,
                material,
                indices,
                transform: Transform::identity(),
                dirty: true,
                buffers: None,
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}
