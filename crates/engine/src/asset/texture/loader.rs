use std::error::Error;
use std::path::Path;

use image::{DynamicImage, GenericImageView};

use super::data::TextureData;
use super::object::TextureObject;
use super::resources::TextureResources;

const SAMPLE_TEXTURE_PNG: &[u8] = include_bytes!("../../assets/happy_tree.png");

pub fn load_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    path: Option<&Path>,
) -> Result<(TextureObject, TextureResources), Box<dyn Error>> {
    let texture_data = match path {
        Some(p) => load_texture_image_from_path(p)?,
        None => load_texture_image_from_bytes(SAMPLE_TEXTURE_PNG)?,
    };

    let size = wgpu::Extent3d {
        width: texture_data.dimensions.0,
        height: texture_data.dimensions.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Diffuse Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texture_data.rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * texture_data.dimensions.0),
            rows_per_image: Some(texture_data.dimensions.1),
        },
        size,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Diffuse Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("diffuse_bind_group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    Ok((
        TextureObject {
            texture,
            view,
            sampler,
        },
        TextureResources {
            bind_group,
            data: texture_data,
        },
    ))
}

pub fn load_texture_image_from_path(path: &Path) -> Result<TextureData, Box<dyn Error>> {
    let bytes = std::fs::read(path)?;
    load_texture_image_from_bytes(&bytes)
}

pub fn load_texture_image_from_bytes(bytes: &[u8]) -> Result<TextureData, Box<dyn Error>> {
    let image: DynamicImage = image::load_from_memory(bytes)?;
    Ok(TextureData {
        dimensions: image.dimensions(),
        rgba: image.to_rgba8(),
    })
}
