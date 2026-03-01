use anyhow::Context;
use engine_i18n::t;
use std::sync::Arc;
use wgpu::{
    Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages,
};
use winit::{dpi::PhysicalSize, window::Window};

pub struct Renderer {
    pub window: Arc<Window>,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub surface_config: SurfaceConfiguration,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Renderer> {
        let instance_desc = &InstanceDescriptor::default();
        let instance = Instance::new(instance_desc);
        let surface = instance
            .create_surface(window.clone())
            .context(t!("renderer.create_surface"))?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context(t!("renderer.request_adapter"))?;
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .context(t!("renderer.request_device"))?;
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface
                .get_capabilities(&adapter)
                .formats
                .first()
                .context(t!("renderer.texture_format_error"))?
                .clone(),
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        Ok(Self {
            window: window,
            surface: surface,
            device: device,
            queue: queue,
            surface_config: surface_config,
        })
    }

    pub fn render(&self) -> anyhow::Result<()> {
        let surface = &self.surface;
        let current_texture = surface
            .get_current_texture()
            .context(t!("renderer.get_surface_texture"))?;
        current_texture.present();
        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}
