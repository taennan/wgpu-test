use crate::{
    game::Mesh,
    graphics::{
        CameraRenderer, MeshInstanceBufferData, RendererUpdateInput,
        bind_group::{BindGroupBuilder, BindGroupLayoutBuilder},
        buffer::MutBuffer,
        geometry::Vertex,
        pipeline::{CreatePipelineInput, PipelinePool, RenderPassDrawer},
    },
    utils::paths,
};
use std::{mem, path::PathBuf, sync::LazyLock};
use wgpu::{
    BindGroup, BindingResource, BindingType, BufferUsages, Device, RenderPass, SamplerBindingType,
    ShaderStages, TextureSampleType, TextureViewDimension,
};

#[derive(Debug)]
pub struct MeshRenderer {
    texture_bind_group: Option<BindGroup>,
    vertex_buffer: Option<MutBuffer>,
    instance_buffer: Option<MutBuffer>,
    index_buffer: Option<MutBuffer>,
    is_inited: bool,
    total_instances: usize,
    total_indices: usize,
}

impl MeshRenderer {
    const SHADER_PATH: LazyLock<PathBuf> = LazyLock::new(|| paths::shader("mesh"));

    pub fn new(
        camera_renderer: &CameraRenderer,
        pipelines: &mut PipelinePool,
        device: &Device,
    ) -> Self {
        let pipeline_key = &*Self::SHADER_PATH;
        if !pipelines.has(pipeline_key) {
            let texture_bind_group_layout = BindGroupLayoutBuilder::new()
                .name("Mesh Texture Bind Group Layout")
                .entry(
                    ShaderStages::FRAGMENT,
                    BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                )
                .entry(
                    ShaderStages::FRAGMENT,
                    BindingType::Sampler(SamplerBindingType::Filtering),
                )
                .build(device);

            pipelines.load(&CreatePipelineInput {
                shader_path: pipeline_key,
                bind_group_layouts: &[
                    camera_renderer.bind_group_layout().clone(),
                    texture_bind_group_layout,
                ],
                vertex_buffer_layouts: &[Vertex::LAYOUT, MeshInstanceBufferData::LAYOUT],
            });
        }

        Self {
            texture_bind_group: None,
            vertex_buffer: None,
            instance_buffer: None,
            index_buffer: None,
            is_inited: false,
            total_instances: 0,
            total_indices: 0,
        }
    }

    pub fn update(&mut self, meshes: &[Mesh], input: &mut RendererUpdateInput) {
        if !self.is_inited && meshes.is_empty() {
            return;
        }

        let textures: Vec<_> = meshes
            .iter()
            .map(|mesh| mesh.texture_path.clone())
            .collect();
        input
            .texture_atlas
            .insert(&textures, input.device, input.queue, input.encoder);

        let vertices: Vec<_> = meshes
            .iter()
            .flat_map(|mesh| input.geometry.load(&mesh.mesh_path).vertices.clone())
            .collect();

        let indices: Vec<_> = meshes
            .iter()
            .flat_map(|mesh| input.geometry.load(&mesh.mesh_path).indices.clone())
            .collect();
        let instance_buffer_data: Vec<_> = meshes
            .iter()
            .map(|mesh| MeshInstanceBufferData::from_mesh(mesh, &input.texture_atlas))
            .collect();

        if !self.is_inited || meshes.len() != self.total_instances {
            let pipeline_key = &*Self::SHADER_PATH;
            let texture_bind_group_layout = input
                .pipelines
                .get(pipeline_key)
                .expect("Mesh pipeline was not loaded")
                .bind_group_layouts
                .get(1)
                .expect("Mesh texture bind group layout was not loaded");
            let texture_bind_group = BindGroupBuilder::new()
                .name("Mesh Texture Bind Group")
                .entry(BindingResource::TextureView(
                    input.texture_atlas.texture_view(),
                ))
                .entry(BindingResource::Sampler(input.texture_atlas.sampler()))
                .build(&texture_bind_group_layout, input.device);
            self.texture_bind_group = Some(texture_bind_group);

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
    }

    pub fn render(
        &self,
        camera_bind_group: &BindGroup,
        render_pass: RenderPass<'_>,
        pipelines: &mut PipelinePool,
    ) {
        let (texture_bind_group, vertex_buffer, instance_buffer, index_buffer) = match (
            &self.texture_bind_group,
            &self.vertex_buffer,
            &self.instance_buffer,
            &self.index_buffer,
        ) {
            (Some(t), Some(v), Some(i), Some(ib)) => (t, v, i, ib),
            _ => return,
        };

        let pipeline = &pipelines
            .get(&*Self::SHADER_PATH)
            .expect("Mesh pipeline was not loaded")
            .pipeline;

        RenderPassDrawer::new()
            .bind_group(camera_bind_group)
            .bind_group(texture_bind_group)
            .vertex_buffer(vertex_buffer.buffer())
            .vertex_buffer(instance_buffer.buffer())
            .index_buffer(index_buffer.buffer(), self.total_indices as u32)
            .instance_range(0..self.total_instances as u32)
            .draw(render_pass, &pipeline);
    }
}
