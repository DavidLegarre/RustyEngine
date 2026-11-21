use wgpu::util::DeviceExt;

use crate::render::vertex::{self, Vertex};

/// Vertex + index buffers and the draw sequence that binds them.
pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }

    pub fn pentagon(device: &wgpu::Device) -> Self {
        Self::new(device, vertex::VERTICES, vertex::INDICES)
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, instances: std::ops::Range<u32>) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.num_indices, 0, instances);
    }
}
