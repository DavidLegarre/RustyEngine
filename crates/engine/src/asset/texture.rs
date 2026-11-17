use image::{DynamicImage, GenericImageView, RgbaImage};
use std::error::Error;
use std::path::Path;

pub struct TextureData {
    pub rgba: RgbaImage,
    pub dimensions: (u32, u32),
}

pub struct TextureObject {
    pub diffuse_texture: wgpu::Texture,
    pub texture_data: TextureData,
}

impl TextureObject {
    pub fn width(&self) -> u32 {
        self.texture_data.dimensions.0
    }

    pub fn height(&self) -> u32 {
        self.texture_data.dimensions.1
    }
    pub fn rgba(&self) -> &RgbaImage {
        &self.texture_data.rgba
    }
}

const SAMPLE_TEXTURE_PNG: &[u8] = include_bytes!("../assets/happy_tree.png");

pub fn load_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: Option<&Path>,
) -> Result<TextureObject, Box<dyn Error>> {
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

    Ok(TextureObject {
        diffuse_texture: texture,
        texture_data,
    })
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
