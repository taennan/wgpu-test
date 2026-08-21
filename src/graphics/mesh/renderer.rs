use crate::{
    game::Mesh,
    graphics::{
        RendererUpdateInput,
        buffer::MutBuffer,
        geometry::Vertex,
        mesh::MeshInstanceBufferData,
        pipeline::{PipelinePool, RenderPassDrawer},
    },
};
use std::{collections::HashSet, mem};
use wgpu::{BindGroup, BufferUsages, RenderPass};

#[derive(Debug, Default)]
pub struct MeshRenderer {
    vertex_buffer: Option<MutBuffer>,
    instance_buffer: Option<MutBuffer>,
    index_buffer: Option<MutBuffer>,
    is_inited: bool,
    total_instances: usize,
    total_indices: usize,
}

impl MeshRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, meshes: &[Mesh], input: &RendererUpdateInput) {
        /*

        if !self.is_inited && meshes.is_empty() {
            return;
        }

        let mesh_paths: HashSet<_> = meshes.iter().map(|mesh| &mesh.mesh_path).collect();
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

        let instance_buffer_data: Vec<_> = meshes
            .iter()
            .map(|mesh| MeshInstanceBufferData::from_mesh(mesh, &input.texture_atlas))
            .collect();

        if !self.is_inited || meshes.len() != self.total_instances {
            let vertex_buffer = MutBuffer::builder()
                .name("Mesh Vertex Buffer")
                .usages(BufferUsages::MAP_WRITE | BufferUsages::VERTEX)
                .build(Vertex::SIZE * vertices.len() as u64, input.device);
            self.vertex_buffer = Some(vertex_buffer);

            let index_buffer = MutBuffer::builder()
                .name("Mesh Index Buffer")
                .usages(BufferUsages::MAP_WRITE | BufferUsages::INDEX)
                .build((mem::size_of::<u32>() * indices.len()) as u64, input.device);
            self.index_buffer = Some(index_buffer);

            let instance_buffer = MutBuffer::builder()
                .name("Mesh Instance Buffer")
                .usages(BufferUsages::MAP_WRITE | BufferUsages::VERTEX)
                .build(
                    MeshInstanceBufferData::SIZE * instance_buffer_data.len() as u64,
                    input.device,
                );
            self.instance_buffer = Some(instance_buffer);
        }

        log::debug!(
            "Writing verts and indices {} {}",
            vertices.len(),
            indices.len()
        );

        self.vertex_buffer.as_mut().map(|b| b.write(&vertices));
        self.index_buffer
            .as_mut()
            .map(|b| b.write(indices.as_slice()));
        self.instance_buffer
            .as_mut()
            .map(|b| b.write(&instance_buffer_data));
        self.total_indices = indices.len();
        self.total_instances = meshes.len();
        self.is_inited = true;

        */
    }

    pub fn render(
        &self,
        camera_bind_group: &BindGroup,
        atlas_bind_group: &BindGroup,
        render_pass: RenderPass<'_>,
        pipelines: &PipelinePool,
    ) {
        let (vertex_buffer, instance_buffer, index_buffer) = match (
            &self.vertex_buffer,
            &self.instance_buffer,
            &self.index_buffer,
        ) {
            (Some(v), Some(i), Some(ib)) => (v, i, ib),
            _ => return,
        };

        RenderPassDrawer::new()
            .bind_groups(&[camera_bind_group, atlas_bind_group])
            .vertex_buffer(vertex_buffer.buffer())
            .vertex_buffer(instance_buffer.buffer())
            .index_buffer(index_buffer.buffer(), self.total_indices as u32)
            .instance_range(0..self.total_instances as u32)
            .draw(render_pass, pipelines.mesh());
    }
}
