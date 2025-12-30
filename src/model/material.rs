use wgpu::{Device, Queue};

use crate::texture::{self, Texture};

pub struct Material {
    pub name: String,

    // CPU buffers
    pub diffuse_image: image::DynamicImage,

    pub dirty: bool, // if true, GPU buffers will be updated
    pub buffers: Option<MaterialBuffers>,
}

/// GPU buffers for a mesh material
pub struct MaterialBuffers {
    #[allow(unused)]
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn update_buffers(
        &mut self,
        device: &Device,
        queue: &Queue,
        layout: &wgpu::BindGroupLayout,
    ) {
        let diffuse_texture =
            Texture::from_image(device, queue, &self.diffuse_image, Some(&self.name)).unwrap();

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

        self.buffers = Some(MaterialBuffers {
            diffuse_texture,
            bind_group,
        });
        self.dirty = false;
    }
}
