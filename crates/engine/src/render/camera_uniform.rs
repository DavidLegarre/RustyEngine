use wgpu::util::DeviceExt;

use crate::render::camera::Camera;

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn from_camera(camera: &Camera, aspect: f32) -> Self {
        Self {
            view_proj: camera.build_view_projection_matrix(aspect).into(),
        }
    }
}

/// GPU side of the camera: uniform buffer + bind group.
/// The pure `Camera` math stays free of wgpu types.
pub struct CameraResources {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    uniform: CameraUniform,
}

impl CameraResources {
    /// Layout the pipeline must declare before it is created.
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        camera: &Camera,
        aspect: f32,
    ) -> Self {
        let uniform = CameraUniform::from_camera(camera, aspect);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group,
            uniform,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, camera: &Camera, aspect: f32) {
        self.uniform = CameraUniform::from_camera(camera, aspect);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
