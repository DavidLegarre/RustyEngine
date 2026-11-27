use std::error::Error;
use std::sync::Arc;

use winit::window::Window;

use crate::asset::texture::{self, TextureObject, TextureResources};
use crate::render::camera::Camera;
use crate::render::camera::CameraResources;
use crate::render::gpu::Gpu;
use crate::render::instance::Instance;
use crate::render::mesh::Mesh;
use crate::render::pipeline;
use crate::render::swapchain::Swapchain;

use cgmath::prelude::*;
use wgpu::util::DeviceExt;

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct Renderer {
    gpu: Gpu,
    swapchain: Swapchain,
    pipeline: wgpu::RenderPipeline,
    mesh: Mesh,
    #[allow(dead_code)] // the bind group samples it; keep the texture alive alongside it
    texture: TextureObject,
    texture_resources: TextureResources,
    camera_resources: CameraResources,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
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

        let (texture, texture_resources) =
            texture::load_texture(gpu.device(), gpu.queue(), &texture_layout, None)?;
        let mesh = Mesh::pentagon(gpu.device());
        let camera_resources = CameraResources::new(
            gpu.device(),
            &camera_layout,
            &Camera::default(),
            swapchain.aspect(),
        );
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can affect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        Ok(Self {
            gpu,
            swapchain,
            pipeline,
            mesh,
            texture,
            texture_resources,
            camera_resources,
            instances,
            instance_buffer,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swapchain.resize(self.gpu.device(), width, height);
    }

    pub fn render(&mut self, camera: &Camera) {
        let Some(texture) = self.swapchain.acquire(self.gpu.device()) else {
            return;
        };

        self.camera_resources
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
            render_pass.set_bind_group(0, &self.texture_resources.bind_group, &[]);
            render_pass.set_bind_group(1, self.camera_resources.bind_group(), &[]);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            self.mesh
                .draw(&mut render_pass, 0..self.instances.len() as u32);
        }
        self.gpu.queue().submit(Some(encoder.finish()));
        self.gpu.queue().present(texture);
    }
}
