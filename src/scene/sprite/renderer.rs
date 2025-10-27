use super::buffer_data::{SpriteInstanceBufferData, SpriteVertexBufferData};
use crate::{
    graphics::{
        RenderPass,
        buffer::mut_buffer::MutBuffer,
        pipeline::{CreatePipelineInput, PipelinePool},
        texture::TexturePool,
    },
    scene::{Sprite, camera::CameraRenderer},
    utils::paths,
};
use std::{path::PathBuf, sync::LazyLock};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages,
    CommandEncoder, Device, SamplerBindingType, ShaderStages, TextureSampleType, TextureView,
    TextureViewDimension,
    util::{BufferInitDescriptor, DeviceExt},
};

pub struct SpriteRenderer {
    bind_group: BindGroup,
    vertex_buffer: Buffer,
    instance_buffer: MutBuffer,
    total_rendered: usize,
}

pub struct SpriteRendererUpdateInput<'a> {
    pub device: &'a Device,
    pub sprites: &'a [&'a Sprite],
}

pub struct SpriteRendererRenderInput<'a> {
    pub camera_bind_group: &'a BindGroup,
    pub texture_view: &'a TextureView,
    pub encoder: &'a mut CommandEncoder,
    pub pipelines: &'a PipelinePool,
}

impl SpriteRenderer {
    const PIPELINE_KEY: LazyLock<PathBuf> = LazyLock::new(|| paths::shader("sprite"));

    pub fn bind_group_layouts<'a>(
        camera_renderer: &'a CameraRenderer,
        device: &Device,
    ) -> (&'a BindGroupLayout, BindGroupLayout) {
        let camera_bind_group_layout = camera_renderer.bind_group_layout();
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Sprite Bind Group Layout"),
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
                    /*
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::VERTEX,
                        count: None,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    },
                     */
                ],
            });

        (camera_bind_group_layout, texture_bind_group_layout)
    }

    pub fn new(
        sprites: &[&Sprite],
        camera_renderer: &CameraRenderer,
        textures: &mut TexturePool,
        pipelines: &mut PipelinePool,
        device: &Device,
    ) -> Self {
        let base_sprite = sprites[0];

        let pipeline_key = &*Self::PIPELINE_KEY;
        let pipeline_data = match pipelines.get(pipeline_key) {
            Some(existing) => existing,
            _ => {
                let bind_group_layouts = Self::bind_group_layouts(camera_renderer, device);
                let pipeline_input = CreatePipelineInput {
                    shader_path: pipeline_key,
                    bind_group_layouts: &[bind_group_layouts.0.clone(), bind_group_layouts.1],
                    vertex_buffer_layouts: &[
                        SpriteVertexBufferData::DESCRIPTOR,
                        SpriteInstanceBufferData::DESCRIPTOR,
                    ],
                };

                pipelines.load(&pipeline_input);
                pipelines.get_unchecked(pipeline_key)
            }
        };

        let bind_group_layout = pipeline_data
            .bind_group_layouts
            .get(1)
            .expect("Sprite BindGroupLayouts not present");

        let texture_path = paths::root().join(&base_sprite.texture_path);
        let texture = textures
            .load(&texture_path)
            .expect("Failed to load Sprite texture file");

        let texture_divisions_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Sprite Texture Divisions Buffer"),
            usage: BufferUsages::VERTEX | BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[base_sprite.texture_divisions]),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Sprite Bind Group"),
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.diffuse_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.diffuse_sampler),
                },
                /*
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(
                        texture_divisions_buffer.as_entire_buffer_binding(),
                    ),
                },
                 */
            ],
        });

        let vertex_buffer = SpriteVertexBufferData::buffer(&device);
        let instance_buffer = Self::init_instance_buffer(sprites, device);
        let total_rendered = sprites.len();

        Self {
            bind_group,
            vertex_buffer,
            instance_buffer,
            total_rendered,
        }
    }

    fn init_instance_buffer(sprites: &[&Sprite], device: &Device) -> MutBuffer {
        let instance_buffer_data = Self::instance_buffer_data(sprites);
        let instance_buffer = MutBuffer::builder()
            .name("Sprite Instance Buffer")
            .usages(BufferUsages::VERTEX | BufferUsages::MAP_WRITE)
            .build_init(&instance_buffer_data, device);

        instance_buffer
    }

    fn instance_buffer_data(sprites: &[&Sprite]) -> Vec<SpriteInstanceBufferData> {
        sprites
            .iter()
            .map(|sprite| SpriteInstanceBufferData::from(*sprite))
            .collect()
    }

    pub fn update(&mut self, input: SpriteRendererUpdateInput) {
        let total_to_render = input.sprites.len();
        let should_recreate_buffer = total_to_render != self.total_rendered;
        if should_recreate_buffer {
            self.instance_buffer = Self::init_instance_buffer(input.sprites, input.device);
            self.total_rendered = total_to_render;
            return;
        }

        let instance_buffer_data = Self::instance_buffer_data(input.sprites);
        self.instance_buffer.write_async(
            &instance_buffer_data,
            || {},
            || log::error!("Didn't write sprite instance buffer"),
        );
    }

    pub fn render(&self, input: SpriteRendererRenderInput) {
        let pipeline_data = input.pipelines.get_unchecked(&*Self::PIPELINE_KEY);
        let render_pass = RenderPass::builder()
            .bind_group(input.camera_bind_group)
            .bind_group(&self.bind_group)
            .vertex_buffer(&self.vertex_buffer)
            .vertex_buffer(&self.instance_buffer.buffer())
            .index_range(0..6)
            .build("Sprite", input.texture_view, &pipeline_data.pipeline);

        render_pass.begin(input.encoder);
    }
}
