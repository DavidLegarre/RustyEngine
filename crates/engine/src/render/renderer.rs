use std::error::Error;
use std::sync::Arc;

use winit::window::Window;

use crate::asset::texture::{self, TextureObject};
use crate::render::camera::Camera;
use crate::render::camera_uniform::CameraResources;
use crate::render::gpu::Gpu;
use crate::render::mesh::Mesh;
use crate::render::pipeline;
use crate::render::swapchain::Swapchain;

pub struct Renderer {
    gpu: Gpu,
    swapchain: Swapchain,
    pipeline: wgpu::RenderPipeline,
    mesh: Mesh,
    texture: TextureObject,
    camera: CameraResources,
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

        // Layouts are the contract: they must exist before the pipeline that uses them.
        let texture_layout = TextureObject::bind_group_layout(gpu.device());
        let camera_layout = CameraResources::bind_group_layout(gpu.device());
        let pipeline = pipeline::create_pipeline(
            gpu.device(),
            swapchain.format(),
            &[Some(&texture_layout), Some(&camera_layout)],
        )?;

        let texture = texture::load_texture(gpu.device(), gpu.queue(), &texture_layout, None)?;
        let mesh = Mesh::pentagon(gpu.device());
        let camera = CameraResources::new(
            gpu.device(),
            &camera_layout,
            &Camera::default(),
            swapchain.aspect(),
        );

        Ok(Self {
            gpu,
            swapchain,
            pipeline,
            mesh,
            texture,
            camera,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swapchain.resize(self.gpu.device(), width, height);
    }

    pub fn render(&mut self, camera: &Camera) {
        let Some(texture) = self.swapchain.acquire(self.gpu.device()) else {
            return;
        };

        self.camera
            .update(self.gpu.queue(), camera, self.swapchain.aspect());

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
            render_pass.set_bind_group(1, self.camera.bind_group(), &[]);
            self.mesh.draw(&mut render_pass);
        }
        self.gpu.queue().submit(Some(encoder.finish()));
        self.gpu.queue().present(texture);
    }
}
