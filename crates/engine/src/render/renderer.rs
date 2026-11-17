use std::error::Error;
use std::sync::Arc;

use winit::window::Window;

use crate::asset::texture::{self, TextureObject};
use crate::render::camera::{Camera, CameraUniform};
use crate::render::gpu::Gpu;
use crate::render::pipeline;
use crate::render::swapchain::Swapchain;
use crate::render::vertex;
use wgpu::util::DeviceExt;

pub struct Renderer {
    gpu: Gpu,
    camera: Camera,
    swapchain: Swapchain,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    texture: TextureObject,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let surface = instance.create_surface(window.clone())?;

        let (gpu, adapter) = Gpu::from_surface(&instance, &surface)?;
        let swapchain = Swapchain::new(surface, &adapter, gpu.device(), window.inner_size())?;

        let vertex_buffer = Self::get_vertex_buffer(gpu.device());
        let index_buffer = Self::get_index_buffer(gpu.device());
        let num_indices = vertex::INDICES.len() as u32;
        let texture_layout = TextureObject::bind_group_layout(gpu.device());
        let pipeline =
            pipeline::create_pipeline(gpu.device(), swapchain.format(), &[Some(&texture_layout)])?;
        let texture = texture::load_texture(gpu.device(), gpu.queue(), &texture_layout, None)?;
        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: swapchain.config.width as f32 / swapchain.config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Ok(Self {
            gpu,
            camera,
            swapchain,
            pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            texture,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swapchain.resize(self.gpu.device(), width, height);
    }

    pub fn render(&self) {
        let Some(texture) = self.swapchain.acquire(self.gpu.device()) else {
            return;
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                ..Default::default()
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.texture.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        self.gpu.queue().submit(Some(encoder.finish()));
        self.gpu.queue().present(texture);
    }

    fn get_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertex::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn get_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(vertex::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        })
    }
}
