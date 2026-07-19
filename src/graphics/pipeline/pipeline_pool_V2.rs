use std::fs;
use wgpu::{
    BindGroupLayout, Device, PipelineCompilationOptions, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat,
    VertexBufferLayout,
};

use crate::{
    graphics::{
        MeshInstanceBufferData, SpriteInstanceBufferData,
        geometry::Vertex,
        tilemap::buffer_data::{TilemapInstanceBufferData, TilemapVertexBufferData},
    },
    utils::paths,
};

pub struct PipelinePool {
    mesh: RenderPipeline,
    sprite: RenderPipeline,
    tilemap: RenderPipeline,
}

pub struct GlobalBindGroupLayouts {
    pub camera: BindGroupLayout,
    pub screen_size: BindGroupLayout,
    pub atlas: BindGroupLayout,
}

impl PipelinePool {
    pub fn new(
        bind_group_layouts: GlobalBindGroupLayouts,
        texture_format: TextureFormat,
        device: &Device,
    ) -> Self {
        Self {
            mesh: Self::create_pipeline(
                "mesh",
                texture_format,
                &[bind_group_layouts.camera, bind_group_layouts.atlas.clone()],
                &[Vertex::LAYOUT, MeshInstanceBufferData::LAYOUT],
                device,
            ),
            sprite: Self::create_pipeline(
                "sprite",
                texture_format,
                &[
                    bind_group_layouts.screen_size.clone(),
                    bind_group_layouts.atlas.clone(),
                ],
                &[Vertex::LAYOUT, SpriteInstanceBufferData::LAYOUT],
                device,
            ),
            tilemap: Self::create_pipeline(
                "tilemap",
                texture_format,
                &[
                    bind_group_layouts.screen_size.clone(),
                    bind_group_layouts.atlas.clone(),
                ],
                &[
                    TilemapVertexBufferData::LAYOUT,
                    TilemapInstanceBufferData::LAYOUT,
                ],
                device,
            ),
        }
    }

    fn create_pipeline<'a>(
        shader_name: &str,
        texture_format: TextureFormat,
        bind_group_layouts: &'a [BindGroupLayout],
        vertex_buffer_layouts: &'a [VertexBufferLayout<'a>],
        device: &Device,
    ) -> RenderPipeline {
        let shader_path = &paths::shader(shader_name);
        log::debug!("Reading shader file {:?}", shader_path);

        let shader_source_bytes = fs::read(&shader_path).expect("Failed to read shader file");
        let shader_source_text = String::from_utf8_lossy(&shader_source_bytes);
        let name = &shader_path.to_str().unwrap_or("Unkown");

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", name)),
            source: ShaderSource::Wgsl(shader_source_text.into()),
        });

        let layouts = bind_group_layouts
            .iter()
            .map(|l| Some(l))
            .collect::<Vec<_>>();
        log::debug!("Creating pipeline layout for {}", name);
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{} Pipeline Layout", name)),
            bind_group_layouts: &layouts,
            immediate_size: 0,
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&format!("{} Pipeline", name)),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                //entry_point: None,
                entry_point: Some("vertex_main"),
                buffers: &vertex_buffer_layouts
                    .iter()
                    .map(|layout| Some(layout.clone()))
                    .collect::<Vec<_>>(),
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                //entry_point: None,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: u64::MAX,
                alpha_to_coverage_enabled: false,
            },
            depth_stencil: None,
            multiview_mask: None,
            cache: None,
        });

        render_pipeline
    }

    pub fn mesh(&self) -> &RenderPipeline {
        &self.mesh
    }

    pub fn sprite(&self) -> &RenderPipeline {
        &self.sprite
    }

    pub fn tilemap(&self) -> &RenderPipeline {
        &self.tilemap
    }
}
