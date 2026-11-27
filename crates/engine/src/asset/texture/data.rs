use image::RgbaImage;

/// CPU-side pixels and dimensions for a texture loaded from disk or memory.
pub struct TextureData {
    pub rgba: RgbaImage,
    pub dimensions: (u32, u32),
}

impl TextureData {
    pub fn width(&self) -> u32 {
        self.dimensions.0
    }

    pub fn height(&self) -> u32 {
        self.dimensions.1
    }

    pub fn rgba(&self) -> &RgbaImage {
        &self.rgba
    }
}
