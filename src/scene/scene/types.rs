use crate::{
    graphics::{
        geometry::GeometryPool,
        pipeline::PipelinePool,
        texture::TexturePool,
        texture_v2::{TextureAtlas, TextureBufferPool},
    },
    scene::Scene,
};
use wgpu::{CommandEncoder, Device, TextureView};

pub struct RendererUpdateInput<'a> {
    pub texture_atlas: &'a mut TextureAtlas,
    pub textures: &'a mut TexturePool,
    pub texture_buffers: &'a mut TextureBufferPool,
    pub geometry: &'a mut GeometryPool,
    pub pipelines: &'a mut PipelinePool,
    pub device: &'a Device,
    pub encoder: &'a mut CommandEncoder,
}

pub struct RendererRenderInput<'a> {
    pub texture_view: &'a TextureView,
    pub pipelines: &'a mut PipelinePool,
    pub encoder: &'a mut CommandEncoder,
}
