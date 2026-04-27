use std::sync::{Arc, RwLock};

use wgpu::util::DeviceExt as _;
use winit::dpi::PhysicalSize;

use crate::{
    camera::CameraUniform,
    model::{TransformRaw, VertexDesc as _},
    texture::{self, Texture},
    Camera, Scene, Vertex,
};

/// Struct containing all bind groups and layouts for the main render pipeline
pub(crate) struct MainRenderPipeline {
    device: wgpu::Device,
    queue: wgpu::Queue,
    output_texture: texture::Texture,
    pub pipeline: wgpu::RenderPipeline,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub depth_texture: crate::texture::Texture,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}
impl MainRenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_texture: &texture::Texture,
    ) -> Self {
        // how should textures be bound to the shader
        let texture_bind_group_layout = Self::create_texture_bind_group_layout(&device);

        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = Self::create_camera_bind_group_layout(&device);

        let camera_bind_group =
            Self::create_camera_bind_group(&device, &camera_bind_group_layout, &camera_buffer);

        let pipeline = Self::create_render_pipeline(
            &device,
            output_texture,
            &texture_bind_group_layout,
            &camera_bind_group_layout,
        );

        let depth_texture =
            Texture::create_depth_texture(&device, &output_texture.texture, "depth_texture");

        Self {
            device: device.clone(),
            queue: queue.clone(),
            output_texture: output_texture.clone(),
            pipeline,
            camera_bind_group,
            camera_uniform,
            camera_buffer,
            depth_texture,
            texture_bind_group_layout,
        }
    }

    pub fn render_pass(
        &mut self,
        size: PhysicalSize<u32>,
        encoder: &mut wgpu::CommandEncoder,
        scene: &Scene,
    ) {
        if let Some((_id, mut cam)) = scene.find_first_component::<Camera>() {
            if let Ok(mut cam) = cam.write() {
                cam.update_aspect(size.width as f32, size.height as f32);
                self.camera_uniform.view_proj = cam.get_view_projection_matrix().into();
            }
        } else {
            panic!("No camera in scene!");
        }

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Initial Render Pass"),
            color_attachments: &[
                // corresponds to @location(0) in wgsl shader
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.output_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.pipeline);

        // render scene
        scene
            .draw_scene(
                &mut render_pass,
                &self.device,
                &self.queue,
                &self.camera_bind_group,
                &self.texture_bind_group_layout,
            )
            .expect("couldn't draw mesh");
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        output_texture: &texture::Texture,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("render_shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(texture_bind_group_layout),
                    Some(camera_bind_group_layout),
                ],
                immediate_size: 0,
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), TransformRaw::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_texture.texture.format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // allow transparent textures
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to line shows wireframes
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // how to write to depth texture
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multiview_mask: None,
            cache: None, // only useful for andriod compilation targets
        })
    }

    fn create_camera_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        })
    }

    fn create_camera_bind_group(
        device: &wgpu::Device,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        camera_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        })
    }

    fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }
}
