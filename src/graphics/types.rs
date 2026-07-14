use crate::graphics::{geometry::GeometryPool, texture::TextureAtlas};
use wgpu::Device;

pub struct RendererUpdateInput<'a> {
    pub texture_atlas: &'a mut TextureAtlas,
    pub geometry: &'a mut GeometryPool,
    //pub pipelines: &'a mut PipelinePool,
    pub device: &'a Device,
    //pub queue: &'a mut Queue,
    //pub encoder: &'a mut CommandEncoder,
}

use glam::UVec2;

fn test() {
    let screen_size = UVec2::new(100, 40);
    let window_size = UVec2::new(100, 100);
}
