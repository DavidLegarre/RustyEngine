//! Headless GPU tests for the render contract: Vertex struct ↔ VertexBufferLayout ↔ WGSL,
//! plus the index buffer path.

use engine::asset::texture::{self, TextureObject};
use engine::render::gpu::Gpu;
use engine::render::pipeline;
use engine::render::vertex;
use wgpu::util::DeviceExt;

#[test]
fn pentagon_renders_from_vertex_and_index_buffers() {
    let gpu = Gpu::headless().expect("no compatible GPU found for headless test");
    let device = gpu.device();
    let queue = gpu.queue();

    let texture_layout = TextureObject::bind_group_layout(device);
    let pipeline = pipeline::create_pipeline(
        device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &[Some(&texture_layout)],
    )
    .expect("pipeline creation failed");
    let texture = texture::load_texture(device, queue, &texture_layout, None)
        .expect("texture loading failed");
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertex::VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(vertex::INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    const SIZE: u32 = 64;
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
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..vertex::INDICES.len() as u32, 0, 0..1);
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

    let px = |x: u32, y: u32| {
        let i = ((y * SIZE + x) * 4) as usize;
        [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
    };

    // Outside the pentagon: background stays black.
    let bg = px(2, 2);
    assert!(
        bg[0] < 10 && bg[1] < 10 && bg[2] < 10,
        "background not black: {bg:?}"
    );

    // Pentagon center sits at uv (0.5, 0.5), so the fragment color must come from
    // the middle of the diffuse texture. Allow for linear filtering between texels.
    let expected = texture
        .rgba()
        .get_pixel(texture.width() / 2, texture.height() / 2)
        .0;
    let center = px(32, 32);
    assert!(
        center != [0, 0, 0, 255],
        "pentagon center not drawn: {center:?}"
    );
    for channel in 0..3 {
        let diff = center[channel].abs_diff(expected[channel]);
        assert!(
            diff <= 60,
            "channel {channel}: rendered {:?} vs texture {expected:?}",
            center
        );
    }
}
