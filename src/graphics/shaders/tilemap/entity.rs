use wgpu::Device;

use super::{attributes::TilemapAttributes, renderer::TilemapRenderer};
use crate::graphics::texture::TexturePool;
use std::path::{Path, PathBuf};

pub struct Tilemap<'a> {
    pub attributes: TilemapAttributes,
    renderer: TilemapRenderer<'a>,
}

impl<'a> Tilemap<'a> {
    pub fn new<P>(
        texture_path: P,
        texture_pool: &mut TexturePool,
        chunk_size: u16,
        device: &Device,
    ) -> Self
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let texture_key = &texture_path;
        texture_pool.load(&texture_key).expect("TODO");
        let texture = texture_pool.get(texture_key).expect("TODO");

        let attributes = TilemapAttributes::new(texture_path, chunk_size, 0);
        let renderer = TilemapRenderer::new(&attributes, texture, device);
        Self {
            attributes,
            renderer,
        }
    }
}
