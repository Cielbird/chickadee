use wgpu::util::DeviceExt as _;
use winit::dpi::PhysicalSize;

use crate::{
    camera::CameraUniform,
    model::{TransformRaw, VertexDesc as _},
    render::fps_indicator::FpsIndicator,
    texture::{self, Texture},
    Camera, Scene, Vertex,
};
use crate::{error::*, Model, TransformComponent};

/// Struct containing all bind groups and layouts for the main render pipeline
pub(crate) struct MainRenderPipeline {
    device: wgpu::Device,
    queue: wgpu::Queue,
    output_texture: texture::Texture,
    pub pipeline: wgpu::RenderPipeline,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_buffer: wgpu::Buffer,
    pub depth_texture: crate::texture::Texture,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    fps_indicator: FpsIndicator,
}
impl MainRenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        output_texture: &texture::Texture,
    ) -> Self {
        // how should textures be bound to the shader
        let texture_bind_group_layout = Self::create_texture_bind_group_layout(device);

        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = Self::create_camera_bind_group_layout(device);

        let camera_bind_group =
            Self::create_camera_bind_group(device, &camera_bind_group_layout, &camera_buffer);

        let pipeline = Self::create_render_pipeline(
            device,
            output_texture,
            &texture_bind_group_layout,
            &camera_bind_group_layout,
        );

        let depth_texture =
            Texture::create_depth_texture(device, &output_texture.texture, "depth_texture");

        let fps_indicator = FpsIndicator::new(device, &output_texture.texture, queue);

        Self {
            device: device.clone(),
            queue: queue.clone(),
            output_texture: output_texture.clone(),
            pipeline,
            camera_bind_group,
            camera_buffer,
            depth_texture,
            texture_bind_group_layout,
            fps_indicator,
        }
    }

    pub fn render_pass(
        &mut self,
        size: PhysicalSize<u32>,
        encoder: &mut wgpu::CommandEncoder,
        scene: &mut Scene,
    ) {
        let camera_uniform;
        {
            let camera = scene
                .get_mut_first_component::<Camera>()
                .expect("No camera in scene!");
            camera.update_aspect(size.width as f32, size.height as f32);
            camera_uniform = CameraUniform {
                view_proj: camera.get_view_projection_matrix().into(),
            };
        }
        let camera = scene
            .get_ref_first_component::<Camera>()
            .expect("No camera in scene!");
        let camera = camera.clone();

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        {
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
            Self::draw_scene(
                scene,
                &mut render_pass,
                &camera,
                &self.device,
                &self.queue,
                &self.camera_bind_group,
                &self.texture_bind_group_layout,
            )
            .expect("couldn't draw mesh");
        }

        {
            // Debug fps indicator TODO this should later become a UI pass
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output_texture.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.fps_indicator
                .draw(&mut rpass, &self.device, &self.queue);
        }
    }

    fn draw_scene(
        scene: &mut Scene,
        render_pass: &mut wgpu::RenderPass,
        camera: &Camera,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // TODO i'd love to find a way to nuke these arguments
        camera_bind_group: &wgpu::BindGroup,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Result<()> {
        // iterate on all components, render renderable components
        for entity_id in scene.entities() {
            let entity = scene.get_entity(&entity_id).unwrap();

            for component_id in &entity.components.clone() {
                if scene.get_ref_component::<Model>(component_id).is_none() {
                    continue;
                }
                // this component is a model.

                let transform_id = scene.get_transform(&entity_id);

                let (model, transform) = scene
                    .get_mut_disjoint_2::<Model, TransformComponent>([component_id, &transform_id]);
                let model = model.unwrap();
                let transform = transform.unwrap();

                Self::draw_model(
                    model,
                    &transform,
                    camera,
                    device,
                    queue,
                    render_pass,
                    camera_bind_group,
                    texture_layout,
                )?;
            }
        }
        Ok(())
    }


    pub fn draw_model(
        model: &mut Model,
        transform: &TransformComponent,
        camera: &Camera,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Result<()> {
        for i in 0..model.meshes.len() {
            Self::draw_mesh(
                model,
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

    pub fn draw_mesh(
        model: &mut Model,
        mesh_index: usize,
        transform: &TransformComponent,
        camera: &Camera,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass,
        camera_bind_group: &wgpu::BindGroup,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Result<()> {
        let mesh = model
            .meshes
            .get_mut(mesh_index)
            .ok_or(Error::Other("Invalid mesh index".to_string()))?;

        // don't render meshes outside of camera view
        if !mesh.is_in_view(&transform.global(), camera) {
            return Ok(());
        }

        let material = model
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
