use std::sync::{Arc, RwLock};

use pollster::FutureExt;
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::render::{
    fps_indicator::{self, FpsIndicator},
    main_pipeline::MainRenderPipeline,
    post_pipeline::PostProcessingPipeline,
};

use super::super::{scene::Scene, texture};

pub struct Renderer<'a> {
    window: Arc<Window>,
    size: PhysicalSize<u32>,

    scene: Arc<RwLock<Scene>>,
    fps_indicator: FpsIndicator,

    surface: wgpu::Surface<'a>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: MainRenderPipeline,
    post_proc_pipeline: PostProcessingPipeline,
}

impl<'a> Renderer<'a> {
    pub fn new(window: Arc<Window>, scene: Arc<RwLock<Scene>>) -> Self {
        let window = window;
        let scene = scene;

        let size = window.inner_size();
        let instance = Self::create_gpu_instance();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = Self::create_adapter(instance, &surface);
        let (device, queue) = Self::create_device(&adapter);
        let surface_caps = surface.get_capabilities(&adapter);
        let config = Self::create_surface_config(size, surface_caps);
        surface.configure(&device, &config);

        // post-processing pipeline
        let post_proc_pipeline = PostProcessingPipeline::new(&device, &queue, &config);

        let render_pipeline =
            MainRenderPipeline::new(&device, &queue, &post_proc_pipeline.input_texture);

        let fps_indicator = FpsIndicator::new(&device, &config, &queue);

        Self {
            window,
            scene,
            fps_indicator,

            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            post_proc_pipeline,
        }
    }

    fn create_surface_config(
        size: PhysicalSize<u32>,
        capabilities: wgpu::SurfaceCapabilities,
    ) -> wgpu::SurfaceConfiguration {
        let surface_format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    fn create_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        let required_features = wgpu::Features::empty() | wgpu::Features::POLYGON_MODE_LINE; // for wireframe rendering
        adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features,
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .block_on()
            .unwrap()
    }

    fn create_adapter(instance: wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .unwrap()
    }

    fn create_gpu_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: Default::default(),
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;

        self.config.width = new_size.width;
        self.config.height = new_size.height;

        self.surface.configure(&self.device, &self.config);
    }

    pub fn try_render(&mut self) -> Result<(), ()> {
        let x = self.surface.get_current_texture();
        match x {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.render(surface_texture).unwrap();
            }
            wgpu::CurrentSurfaceTexture::Timeout => {
                eprintln!("Surface timed out, trying again...");
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                // surface is hidden or minimized
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                todo!("Handle changed surface config !");
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                todo!("Handle lost surface !");
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                panic!("Validation error getting surface texture");
            }
        }
        Ok(())
    }

    pub fn render(&mut self, surf_tex: wgpu::SurfaceTexture) -> Result<(), ()> {
        let surface_view = surf_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // main render pass, locks the scene
            let scene = self.scene.read().unwrap();
            self.render_pipeline
                .render_pass(self.size, &mut encoder, &scene);
        }

        // post processing render pass
        self.post_proc_pipeline.render_pass(&mut encoder, &surface_view);

        self.queue.submit(std::iter::once(encoder.finish()));
        surf_tex.present();

        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
