use image::{DynamicImage, GenericImageView, RgbaImage};
use std::error::Error;
use std::path::Path;

pub struct TextureData {
    pub rgba: RgbaImage,
    pub dimensions: (u32, u32),
}

pub fn load_texture_from_path(path: &Path) -> Result<TextureData, Box<dyn Error>> {
    let bytes = std::fs::read(path)?;
    load_texture_from_bytes(&bytes)
}

pub fn load_texture_from_bytes(bytes: &[u8]) -> Result<TextureData, Box<dyn Error>> {
    let image: DynamicImage = image::load_from_memory(bytes)?;
    Ok(TextureData {
        dimensions: image.dimensions(),
        rgba: image.to_rgba8(),
    })
}

/// Sample asset used while following the tutorial. Path is relative to the
/// workspace root, so it only resolves when run via `cargo run` from there.
pub fn load_sample_texture() -> Result<TextureData, Box<dyn Error>> {
    load_texture_from_path(Path::new("crates/engine/src/assets/happy_tree.png"))
}
