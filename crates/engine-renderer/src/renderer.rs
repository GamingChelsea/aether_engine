use anyhow::Context;
use bevy_ecs::resource::Resource;
use bytemuck::{Pod, Zeroable};
use engine_i18n::t;
use glam::Vec3;
use std::sync::Arc;
use wgpu::{
    BlendState, Buffer, Color, ColorTargetState, ColorWrites, Device, DeviceDescriptor,
    FragmentState, Instance, InstanceDescriptor, MultisampleState, PrimitiveState,
    PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages,
    VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode, include_wgsl,
    wgt::{CommandEncoderDescriptor, TextureViewDescriptor},
};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Resource)]
pub struct Renderer {
    pub window: Arc<Window>,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub surface_config: SurfaceConfiguration,
    pub instance_buffer: Option<Buffer>,
    pub render_pipeline: RenderPipeline,
    pub instance_count: u32,
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

        let shader_modlue = device.create_shader_module(include_wgsl!("shaders/shader.wgsl"));
        let instance_layout = VertexBufferLayout {
            array_stride: (size_of::<InstanceData>() as u64),
            step_mode: VertexStepMode::Instance,
            attributes: &[VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        };
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Basic Render Pipeline"),
            layout: None,
            vertex: VertexState {
                module: &shader_modlue,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[instance_layout],
            },
            fragment: Some(FragmentState {
                module: &shader_modlue,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    blend: Some(BlendState::ALPHA_BLENDING),
                    format: surface_config.format,
                    write_mask: ColorWrites::all(),
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            multisample: MultisampleState::default(),
            depth_stencil: None,
            cache: None,
            multiview_mask: None,
        });

        Ok(Self {
            window: window,
            surface: surface,
            device: device,
            queue: queue,
            surface_config: surface_config,
            instance_buffer: None,
            render_pipeline: pipeline,
            instance_count: 0,
        })
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let surface = &self.surface;
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let current_texture = match surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Timeout) => return Ok(()),
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.surface_config);
                return Ok(());
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                anyhow::bail!("{}", t!("renderer.out_of_memory"));
            }
            Err(e) => {
                anyhow::bail!("wgpu SurfaceError: {:?}", e);
            }
        };

        let view = current_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let color = Color {
            r: 0.0,
            g: 0.2,
            b: 0.25,
            a: 1.0,
        };

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(&self.render_pipeline);

            if self.instance_count > 0 {
                if let Some(instance_buffer) = &self.instance_buffer {
                    render_pass.set_vertex_buffer(0, instance_buffer.slice(..));
                    render_pass.draw(0..3, 0..self.instance_count);
                }
            }
        }

        let cmd_buffer = encoder.finish();

        self.queue.submit(std::iter::once(cmd_buffer));
        current_texture.present();
        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn update_instances(&mut self, data: &[InstanceData]) {
        self.instance_count = data.len() as u32;
        let needed_size = (data.len() * std::mem::size_of::<InstanceData>()) as u64;

        let needs_resize = match &self.instance_buffer {
            None => true,
            Some(buffer) => buffer.size() < needed_size,
        };

        if needs_resize && needed_size > 0 {
            let new_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: needed_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_buffer = Some(new_buffer);
        }
        if let Some(buffer) = &self.instance_buffer {
            if !data.is_empty() {
                self.queue
                    .write_buffer(buffer, 0, bytemuck::cast_slice(data));
            }
        }
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct InstanceData {
    pub position: Vec3,
}
