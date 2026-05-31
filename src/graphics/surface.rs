use crate::error::*;
use glam::UVec2;
use wgpu::{Adapter, Surface, SurfaceConfiguration};

pub struct SurfaceConfigFactory<'a> {
    surface: &'a Surface<'a>,
    adapter: &'a Adapter,
    size: UVec2,
}

impl<'a> SurfaceConfigFactory<'a> {
    pub fn new(surface: &'a Surface<'a>, adapter: &'a Adapter, size: UVec2) -> Self {
        Self {
            surface,
            adapter,
            size,
        }
    }

    pub fn try_build(&self) -> Result<SurfaceConfiguration> {
        let surface_capabilities = self.surface.get_capabilities(self.adapter);
        let surface_formats = surface_capabilities.formats;
        let srgb_formats: Vec<_> = surface_formats
            .iter()
            .filter(|format| format.is_srgb())
            .copied()
            .collect();
        let surface_format = srgb_formats.first().unwrap_or(
            surface_formats
                .first()
                .ok_or(Error::SurfaceCreationFailed)?,
        );
        let surface_alpha_mode = surface_capabilities
            .alpha_modes
            .first()
            .ok_or(Error::SurfaceCreationFailed)?;
        let surface_present_mode = surface_capabilities
            .present_modes
            .first()
            .ok_or(Error::SurfaceCreationFailed)?;
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            width: self.size.x,
            height: self.size.y,
            format: *surface_format,
            alpha_mode: *surface_alpha_mode,
            present_mode: *surface_present_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(config)
    }
}
