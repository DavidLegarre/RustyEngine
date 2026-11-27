use super::data::TextureData;

/// The parts of a loaded texture that are not the GPU texture itself: the
/// bind group used to sample it, plus the CPU-side pixels it was built from.
pub struct TextureResources {
    pub bind_group: wgpu::BindGroup,
    pub data: TextureData,
}
