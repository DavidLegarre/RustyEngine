use std::error::Error;

pub struct Swapchain {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    #[allow(dead_code)] // depth chapter arrives here, not in Renderer
    depth: Option<wgpu::Texture>,
}

impl Swapchain {
    pub fn new(
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> Result<Self, Box<dyn Error>> {
        let config = surface
            .get_default_config(adapter, size.width, size.height)
            .ok_or("Failed to get default surface configuration")?;
        surface.configure(device, &config);

        Ok(Self {
            surface,
            config,
            depth: None,
        })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Width / height. A minimized window reports height 0, which would make the
    /// projection matrix NaN, so fall back to square.
    pub fn aspect(&self) -> f32 {
        if self.config.height == 0 {
            1.0
        } else {
            self.config.width as f32 / self.config.height as f32
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(device, &self.config);
    }

    pub fn acquire(&self, device: &wgpu::Device) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => Some(t),
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => Some(t),
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(device, &self.config);
                None
            }
            _ => None,
        }
    }
}
