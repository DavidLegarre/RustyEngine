//! Headless GPU tests for the render contract: Vertex struct ↔ VertexBufferLayout ↔ WGSL.

use super::renderer::Renderer;
use super::vertex;

#[test]
fn vertex_colors_come_from_the_vertex_buffer() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("no compatible GPU adapter found for headless test");
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("no compatible GPU device found for headless test");

    let pipeline = Renderer::create_pipeline(&device, wgpu::TextureFormat::Rgba8UnormSrgb)
        .expect("pipeline creation failed");
    let vertex_buffer = Renderer::create_triangle_buffer(&device);

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
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertex::VERTICES.len() as u32, 0..1);
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

    // Outside the triangle: background stays black.
    let bg = px(2, 2);
    assert!(
        bg[0] < 10 && bg[1] < 10 && bg[2] < 10,
        "background not black: {bg:?}"
    );

    // Near the top vertex, which is red in VERTICES.
    let top = px(32, 18);
    assert!(
        top[0] > 150 && top[1] < 100 && top[2] < 100,
        "top vertex region not red: {top:?}"
    );

    // Triangle centroid: three vertex colors interpolate to ~equal channels.
    let mid = px(32, 37);
    let lo = mid[0].min(mid[1]).min(mid[2]);
    let hi = mid[0].max(mid[1]).max(mid[2]);
    assert!(lo > 100 && hi < 200, "centroid not ~1/3 grey: {mid:?}");
    assert!(
        hi - lo < 30,
        "centroid channels diverge, vertex colors not interpolated: {mid:?}"
    );
}
