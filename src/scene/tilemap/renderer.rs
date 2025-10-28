use super::{attributes::TilemapAttributes, vertices::TilemapVertices};
use crate::graphics::{
    pipeline::{PipelinePool, RenderPassDrawInput},
    texture::TexturePool,
};
use std::{mem, num::NonZero, path::PathBuf};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, BufferUsages,
    CommandEncoder, Device, SamplerBindingType, ShaderStages, TextureSampleType, TextureView,
    TextureViewDimension,
    util::{BufferInitDescriptor, DeviceExt},
};

pub struct TilemapRenderer {
    texture_bind_group: BindGroup,
}

pub struct TilemapRenderInput<'a> {
    pub attributes: &'a TilemapAttributes,
    pub camera_bind_group: &'a BindGroup,
    pub texture_view: &'a TextureView,
    pub encoder: &'a mut CommandEncoder,
    pub device: &'a Device,
    pub pipeline_pool: &'a PipelinePool,
}

// TODO: This should be absolute
const SHADER_PATH: &'static str = "graphics/shaders/tilemap/shader_v2.wgsl";

impl TilemapRenderer {
    pub fn bind_group_layouts(device: &Device) -> (BindGroupLayout, BindGroupLayout) {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Tilemap Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        count: None,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        count: None,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        count: None,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                NonZero::new((mem::size_of::<u32>() * 2) as u64)
                                    .expect("Failed to create non-zero size"),
                            ),
                        },
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        count: None,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: Some(
                                NonZero::new((mem::size_of::<f32>() * 2) as u64)
                                    .expect("Failed to create non-zero size"),
                            ),
                        },
                    },
                ],
            });

        let camera_bind_group_layout = todo!();

        (camera_bind_group_layout, texture_bind_group_layout)
    }

    pub fn new(
        attributes: &TilemapAttributes,
        texture_pool: &mut TexturePool,
        pipelines: &PipelinePool,
        device: &Device,
    ) -> Self {
        let pipeline_data = pipelines.get_unchecked(&PathBuf::from(SHADER_PATH));
        let bind_group_layout = match pipeline_data.bind_group_layouts.get(0) {
            Some(layout) => layout,
            None => panic!("Bind group layout not found"),
        };

        texture_pool
            .load(&attributes.texture_path())
            .expect("Failed to load Tilemap texture file");
        let texture = texture_pool
            .get(&attributes.texture_path())
            .expect("Failed to get Tilemap texture");

        let texture_tilecount = attributes.texture_tilecount();
        let texture_tilecount_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Tilemap Tilecount Buffer"),
            contents: bytemuck::cast_slice(&[texture_tilecount.x, texture_tilecount.y]),
            usage: BufferUsages::STORAGE,
        });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Tilemap Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.diffuse_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.diffuse_sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: texture_tilecount_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: texture_tilecount_buffer.as_entire_binding(),
                },
            ],
        });

        Self { texture_bind_group }
    }

    pub fn render(&self, input: TilemapRenderInput) {
        let shader_path = PathBuf::from(SHADER_PATH);
        let pipeline_data = input.pipeline_pool.get_unchecked(&shader_path);
        let vertex_buffer = TilemapVertices::new().vertex_buffer(input.attributes, input.device);

        /*
         *
        RenderPassBuilder::new()
            .name("Tilemap")
            .bind_group(input.camera_bind_group)
            .bind_group(&self.texture_bind_group)
            .vertex_buffer(&vertex_buffer)
            .draw(RenderPassDrawInput {
                encoder: input.encoder,
                render_pipeline: &pipeline_data.pipeline,
                texture_view: input.texture_view,
            });
         */
    }
}
