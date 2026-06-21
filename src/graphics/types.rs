use crate::graphics::{geometry::GeometryPool, pipeline::PipelinePool, texture::TextureAtlas};
use wgpu::{CommandEncoder, Device, Queue};

pub struct RendererUpdateInput<'a> {
    pub texture_atlas: &'a mut TextureAtlas,
    pub geometry: &'a mut GeometryPool,
    pub pipelines: &'a mut PipelinePool,
    pub device: &'a Device,
    pub queue: &'a mut Queue,
    pub encoder: &'a mut CommandEncoder,
}
