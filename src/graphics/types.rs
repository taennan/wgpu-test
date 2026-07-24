use crate::graphics::{geometry::GeometryPool, texture::TextureAtlas};
use wgpu::Device;

pub struct RendererUpdateInput<'a> {
    pub texture_atlas: &'a TextureAtlas,
    pub geometry: &'a GeometryPool,
    //pub pipelines: &'a mut PipelinePool,
    pub device: &'a Device,
    //pub queue: &'a mut Queue,
    //pub encoder: &'a mut CommandEncoder,
}
