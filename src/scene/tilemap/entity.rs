use super::{attributes::TilemapAttributes, renderer::TilemapRenderer};
use crate::graphics::{pipeline::PipelinePool, texture::TexturePool};
use glam::UVec2;
use std::path::{Path, PathBuf};
use wgpu::Device;

pub struct Tilemap {
    pub attributes: TilemapAttributes,
    pub renderer: TilemapRenderer,
}

impl Tilemap {
    pub fn new<P>(
        texture_path: P,
        texture_tilecount: UVec2,
        texture_pool: &mut TexturePool,
        pipelines: &PipelinePool,
        chunk_size: u16,
        device: &Device,
    ) -> Self
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let default_tile = 0;
        let attributes =
            TilemapAttributes::new(texture_path, texture_tilecount, chunk_size, default_tile);
        let renderer = TilemapRenderer::new(&attributes, texture_pool, pipelines, device);

        Self {
            attributes,
            renderer,
        }
    }
}
