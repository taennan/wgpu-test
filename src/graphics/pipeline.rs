use crate::{utils, vertex::Vertex};
use std::{fs, path::Path};
use wgpu::{
    BindGroupLayout, Device, PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, TextureFormat,
};

pub struct Pipeline {
    pipeline: RenderPipeline,
}

impl Pipeline {
    pub fn new<P>(
        name: &'static str,
        shader_path: P,
        device: &Device,
        texture_format: TextureFormat,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> Self
    where
        P: AsRef<Path>,
    {
        let filepath = utils::paths::shaders()
            .join(shader_path)
            .join("shader.wgsl");
        let shader_source_bytes = fs::read(filepath).expect("Failed to read shader file");
        let shader_source_text = String::from_utf8_lossy(&shader_source_bytes);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", name)),
            source: ShaderSource::Wgsl(shader_source_text.into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{} Render Pipeline Layout", name)),
            bind_group_layouts,
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&format!("{} Render Pipeline", name)),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &[Vertex::buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
            multiview: None,
            cache: None,
        });

        Pipeline {
            pipeline: render_pipeline,
        }
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }
}
