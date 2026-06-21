use crate::{
    game::Sprite,
    graphics::{
        RendererUpdateInput, SpriteInstanceBufferData,
        bind_group::{BindGroupBuilder, BindGroupLayoutBuilder},
        buffer::mut_buffer::MutBuffer,
        geometry::Vertex,
        pipeline::{CreatePipelineInput, PipelinePool, RenderPassDrawer},
        texture::TextureAtlas,
    },
    utils::paths,
};
use glam::UVec2;
use std::{collections::HashSet, mem, path::PathBuf, sync::LazyLock};
use wgpu::{
    BindGroup, BindingResource, BindingType, BufferBindingType, BufferUsages, Device, RenderPass,
    ShaderStages,
};

pub struct SpriteRenderer {
    screen_size_buffer: MutBuffer,
    screen_size_bind_group: BindGroup,
    vertex_buffer: Option<MutBuffer>,
    instance_buffer: Option<MutBuffer>,
    index_buffer: Option<MutBuffer>,
    total_instances: usize,
    total_indices: usize,
}

impl SpriteRenderer {
    const SHADER_PATH: LazyLock<PathBuf> = LazyLock::new(|| paths::shader("sprite"));

    pub fn new(atlas: &TextureAtlas, pipelines: &mut PipelinePool, device: &Device) -> Self {
        let pipeline_key = &*Self::SHADER_PATH;
        if !pipelines.has(pipeline_key) {
            let aspect_bind_group_layout = BindGroupLayoutBuilder::new()
                .entry(
                    ShaderStages::VERTEX,
                    BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                )
                .build(device);
            pipelines.load(&CreatePipelineInput {
                shader_path: pipeline_key,
                bind_group_layouts: &[aspect_bind_group_layout, atlas.bind_group_layout().clone()],
                vertex_buffer_layouts: &[Vertex::LAYOUT, SpriteInstanceBufferData::LAYOUT],
            });
        }

        let screen_size_bind_group_layout = pipelines
            .get(pipeline_key)
            .and_then(|p| p.bind_group_layouts.get(0))
            .expect("Sprite bind group layouts not loaded");
        let screen_size_buffer = MutBuffer::builder()
            .name("Sprite Aspect Buffer")
            .usages(BufferUsages::VERTEX | BufferUsages::UNIFORM | BufferUsages::MAP_WRITE)
            .build(mem::size_of::<UVec2>() as u64, device);
        let screen_size_bind_group = BindGroupBuilder::new()
            .entry(BindingResource::Buffer(
                screen_size_buffer.buffer().as_entire_buffer_binding(),
            ))
            .build(screen_size_bind_group_layout, device);

        Self {
            screen_size_buffer,
            screen_size_bind_group,
            vertex_buffer: None,
            instance_buffer: None,
            index_buffer: None,
            total_instances: 0,
            total_indices: 0,
        }
    }

    pub fn update_screen_size(&mut self, screen_size: UVec2) {
        self.screen_size_buffer.write(&[screen_size]);
    }

    pub fn update(&mut self, sprites: &[Sprite], input: &mut RendererUpdateInput) {
        if !self.is_inited() && sprites.is_empty() {
            return;
        }

        let mesh_paths: HashSet<_> = sprites.iter().map(|sprite| &sprite.mesh_path).collect();
        let geometries = mesh_paths
            .into_iter()
            .map(|mesh_path| input.geometry.get(mesh_path))
            .filter(Option::is_some)
            .map(|geometry| geometry.expect("Failed to filter out Option::none before unwrapping"))
            .collect::<Vec<_>>();

        let mut vertices = Vec::with_capacity(geometries.len());
        let mut indices = Vec::with_capacity(geometries.len());
        for geometry in geometries.into_iter() {
            let offset_indices = geometry
                .indices
                .iter()
                .map(|index| index + vertices.len() as u32)
                .collect::<Vec<_>>();
            indices.extend_from_slice(&offset_indices);
            vertices.extend_from_slice(&geometry.vertices);
        }

        let instance_buffer_data: Vec<_> = sprites
            .iter()
            .map(|sprite| SpriteInstanceBufferData::from_sprite(sprite, input.texture_atlas))
            .collect();

        if !self.is_inited() || sprites.len() != self.total_instances {
            let vertex_buffer = MutBuffer::builder()
                .name("Sprite Vertex Buffer")
                .usages(BufferUsages::MAP_WRITE | BufferUsages::VERTEX)
                .build(Vertex::SIZE * vertices.len() as u64, input.device);
            self.vertex_buffer = Some(vertex_buffer);

            let index_buffer = MutBuffer::builder()
                .name("Sprite Index Buffer")
                .usages(BufferUsages::MAP_WRITE | BufferUsages::INDEX)
                .build((mem::size_of::<u32>() * indices.len()) as u64, input.device);
            self.index_buffer = Some(index_buffer);

            let instance_buffer = MutBuffer::builder()
                .name("Sprite Instance Buffer")
                .usages(BufferUsages::MAP_WRITE | BufferUsages::VERTEX)
                .build(
                    SpriteInstanceBufferData::SIZE * instance_buffer_data.len() as u64,
                    input.device,
                );
            self.instance_buffer = Some(instance_buffer);
        }

        self.vertex_buffer.as_mut().map(|b| b.write(&vertices));
        self.index_buffer
            .as_mut()
            .map(|b| b.write(indices.as_slice()));
        self.instance_buffer
            .as_mut()
            .map(|b| b.write(&instance_buffer_data));
        self.total_indices = indices.len();
        self.total_instances = sprites.len();
    }

    fn is_inited(&self) -> bool {
        self.vertex_buffer.is_some()
            && self.index_buffer.is_some()
            && self.instance_buffer.is_some()
    }

    pub fn render<'a, 'b>(
        &'a self,
        atlas_bind_group: &BindGroup,
        render_pass: RenderPass<'_>,
        pipelines: &'a PipelinePool,
    ) {
        if !self.is_inited() {
            return;
        }

        let error_message = "SpriteRenderer was not initialised before calling render";
        let (vertex_buffer, instance_buffer, index_buffer) = (
            self.vertex_buffer.as_ref().expect(error_message),
            self.instance_buffer.as_ref().expect(error_message),
            self.index_buffer.as_ref().expect(error_message),
        );

        let pipeline = &pipelines
            .get(&*Self::SHADER_PATH)
            .expect("Sprite pipeline was not loaded")
            .pipeline;

        RenderPassDrawer::new()
            .bind_group(&self.screen_size_bind_group)
            .bind_group(&atlas_bind_group)
            .vertex_buffer(vertex_buffer.buffer())
            .vertex_buffer(instance_buffer.buffer())
            .index_buffer(index_buffer.buffer(), self.total_indices as u32)
            .instance_range(0..self.total_instances as u32)
            .draw(render_pass, &pipeline);
    }
}
