use crate::{
    game::Sprite,
    graphics::{
        RendererUpdateInput, SpriteInstanceBufferData,
        buffer::mut_buffer::MutBuffer,
        geometry::Vertex,
        pipeline::{PipelinePool, RenderPassDrawer},
    },
};
use std::{collections::HashSet, mem};
use wgpu::{BindGroup, BufferUsages, RenderPass};

#[derive(Default)]
pub struct SpriteRenderer {
    vertex_buffer: Option<MutBuffer>,
    instance_buffer: Option<MutBuffer>,
    index_buffer: Option<MutBuffer>,
    total_instances: usize,
    total_indices: usize,
}

impl SpriteRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, sprites: &[Sprite], input: &RendererUpdateInput) {
        /*

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

        */
    }

    fn is_inited(&self) -> bool {
        self.vertex_buffer.is_some()
            && self.index_buffer.is_some()
            && self.instance_buffer.is_some()
    }

    pub fn render<'a, 'b>(
        &'a self,
        atlas_bind_group: &BindGroup,
        screen_size_bind_group: &BindGroup,
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

        RenderPassDrawer::new()
            .bind_group(screen_size_bind_group)
            .bind_group(&atlas_bind_group)
            .vertex_buffer(vertex_buffer.buffer())
            .vertex_buffer(instance_buffer.buffer())
            .index_buffer(index_buffer.buffer(), self.total_indices as u32)
            .instance_range(0..self.total_instances as u32)
            .draw(render_pass, pipelines.sprite());
    }
}
