use crate::graphics::{geometry::GeometryPool, pipeline::PipelinePool, texture::TextureAtlas};
use wgpu::{CommandEncoder, Device, Queue, TextureView};

pub struct RendererUpdateInput<'a> {
    pub texture_atlas: &'a mut TextureAtlas,
    pub geometry: &'a mut GeometryPool,
    pub pipelines: &'a mut PipelinePool,
    pub device: &'a Device,
    pub queue: &'a mut Queue,
    pub encoder: &'a mut CommandEncoder,
}

pub struct RendererRenderInput<'a> {
    pub texture_view: &'a TextureView,
    pub atlas: &'a mut TextureAtlas,
    pub pipelines: &'a mut PipelinePool,
    pub encoder: &'a mut CommandEncoder,
}
