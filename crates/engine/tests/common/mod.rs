//! Shared headless render harness: one offscreen pentagon render, read back to pixels.

// Each test binary compiles this module separately and uses a different subset.
#![allow(dead_code)]

use engine::asset::texture::{self, TextureObject};
use engine::render::camera::Camera;
use engine::render::camera::CameraResources;
use engine::render::gpu::Gpu;
use engine::render::instance::Instance;
use engine::render::mesh::Mesh;
use engine::render::pipeline;
use wgpu::util::DeviceExt;

pub const SIZE: u32 = 64;
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

pub struct Frame {
    pixels: Vec<u8>,
}

impl Frame {
    pub fn px(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * SIZE + x) * 4) as usize;
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }

    /// Average x of all non-background pixels, or -1 when nothing was drawn.
    pub fn centroid_x(&self) -> f32 {
        let (mut sum, mut count) = (0.0f32, 0.0f32);
        for y in 0..SIZE {
            for x in 0..SIZE {
                let p = self.px(x, y);
                if p[0] as u32 + p[1] as u32 + p[2] as u32 > 30 {
                    sum += x as f32;
                    count += 1.0;
                }
            }
        }
        if count == 0.0 { -1.0 } else { sum / count }
    }
}

pub fn render_pentagon(camera: &Camera) -> (Frame, TextureObject) {
    let gpu = Gpu::headless().expect("no compatible GPU found for headless test");
    let device = gpu.device();
    let queue = gpu.queue();

    let texture_layout = TextureObject::bind_group_layout(device);
    let camera_layout = CameraResources::bind_group_layout(device);
    let pipeline = pipeline::create_pipeline(
        device,
        FORMAT,
        &[Some(&texture_layout), Some(&camera_layout)],
    )
    .expect("pipeline creation failed");
    let texture = texture::load_texture(device, queue, &texture_layout, None)
        .expect("texture loading failed");
    let mesh = Mesh::pentagon(device);
    let camera_resources = CameraResources::new(device, &camera_layout, camera, 1.0);

    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance Buffer"),
        contents: bytemuck::cast_slice(&[Instance::default().to_raw()]),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen render target"),
        size: wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback buffer"),
        size: (SIZE * SIZE * 4) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("offscreen render pass"),
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
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &texture.diffuse_bind_group, &[]);
        pass.set_bind_group(1, camera_resources.bind_group(), &[]);
        pass.set_vertex_buffer(1, instance_buffer.slice(..));
        mesh.draw(&mut pass, 0..1);
    }
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(SIZE * 4),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = readback.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).expect("map callback send failed")
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device poll failed");
    receiver
        .recv()
        .expect("map callback lost")
        .expect("buffer map failed");
    let mapped = slice.get_mapped_range().expect("map readback buffer");
    let pixels: Vec<u8> = mapped[..].to_vec();
    drop(mapped);
    readback.unmap();

    (Frame { pixels }, texture)
}
