use crate::graphics::{
    RendererUpdateInput,
    buffer::{BufferSlice, MutBuffer},
    pipeline::{PipelinePool, RenderPassDrawer},
};
use std::{collections::HashMap, fmt::Debug, marker::PhantomData, mem, path::PathBuf};
use wgpu::{BindGroup, BufferUsages, CommandEncoder, Device, RenderPass};

#[derive(Debug)]
pub struct Renderer<R, V, I> {
    vertex_buffer: MutBuffer,
    instance_buffer: MutBuffer,
    index_buffer: MutBuffer,
    geometries: HashMap<PathBuf, GeometryData>,
    renderables: HashMap<u64, RenderableData>,
    _renderable_marker: PhantomData<R>,
    _vertex_marker: PhantomData<V>,
    _instance_marker: PhantomData<I>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GeometryData {
    pub vertices_start: u64,
    pub vertices_len: u64,
    pub indices_start: u64,
    pub indices_len: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderableData {
    pub geometry_key: PathBuf,
    pub instance_index: u64,
}

struct BufferedGeometry {
    geometry_key: PathBuf,
    vertices: CopyBytes,
    indices: CopyBytes,
}

struct BufferedInstance {
    pub render_id: u64,
    pub geometry_key: PathBuf,
    pub bytes: CopyBytes,
}

enum CopyBytes {
    Raw(Box<[u8]>),
    Src { start: u64, size: u64 },
}

pub trait Renderable {
    fn render_id(&self) -> u64;
    fn geometry_key(&self) -> PathBuf;
    fn vertex_bytes(&self) -> Box<[u8]>;
    fn index_bytes(&self) -> Box<[u8]>;
    fn instance_bytes(&self) -> Box<[u8]>;
    fn dirty_bytes(&mut self) -> RenderBuffers<Vec<BufferSlice>>;
}

#[derive(Debug)]
pub struct RenderBuffers<T>
where
    T: Debug,
{
    pub vertex: T,
    pub index: T,
    pub instance: T,
}

impl<R, V, I> Renderer<R, V, I>
where
    R: Renderable,
{
    const VERTEX_SIZE: u64 = mem::size_of::<V>() as u64;
    const INDEX_SIZE: u64 = mem::size_of::<u32>() as u64;
    const INSTANCE_SIZE: u64 = mem::size_of::<I>() as u64;

    pub fn new(device: &Device) -> Self {
        let (vertex_buffer, index_buffer, instance_buffer) = Self::init_buffers(1u64, 1, 1, device);
        Self {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            geometries: HashMap::new(),
            renderables: HashMap::new(),
            _renderable_marker: PhantomData,
            _vertex_marker: PhantomData,
            _instance_marker: PhantomData,
        }
    }

    fn init_buffers<L>(
        vertex_buffer_len: L,
        index_buffer_len: L,
        instance_buffer_len: L,
        device: &Device,
    ) -> (MutBuffer, MutBuffer, MutBuffer)
    where
        L: Into<u64>,
    {
        let vertex_buffer_len = vertex_buffer_len.into();
        let vertex_buffer = MutBuffer::builder()
            .name("Renderer Vertex Buffer")
            .usages(BufferUsages::MAP_WRITE | BufferUsages::VERTEX)
            .build(Self::VERTEX_SIZE * vertex_buffer_len, device);

        let index_buffer_len = index_buffer_len.into();
        let index_buffer = MutBuffer::builder()
            .name("Renderer Index Buffer")
            .usages(BufferUsages::MAP_WRITE | BufferUsages::INDEX)
            .build(Self::INDEX_SIZE * index_buffer_len, device);

        let instance_buffer_len = instance_buffer_len.into();
        let instance_buffer = MutBuffer::builder()
            .name("Renderer Instance Buffer")
            .usages(BufferUsages::MAP_WRITE | BufferUsages::VERTEX)
            .build(Self::INSTANCE_SIZE * instance_buffer_len, device);

        (vertex_buffer, index_buffer, instance_buffer)
    }

    pub fn is_tracking(&self, id: u64) -> bool {
        self.renderables.contains_key(&id)
    }

    pub fn track(&mut self, renderables: &[&R], device: &Device, encoder: &mut CommandEncoder) {
        let renderables = renderables
            .iter()
            .map(|r| (r.render_id(), *r))
            .collect::<HashMap<_, _>>()
            .values()
            .map(|r| *r)
            .collect::<Vec<_>>();

        let new_geometry_data = renderables
            .iter()
            .filter(|r| !self.geometries.contains_key(&r.geometry_key()))
            .map(|r| (r.geometry_key(), r.vertex_bytes(), r.index_bytes()))
            .collect::<Vec<_>>();
        let new_instance_data = renderables
            .iter()
            .filter(|r| !self.renderables.contains_key(&r.render_id()))
            .map(|r| (r.render_id(), r.geometry_key(), r.instance_bytes()))
            .collect::<Vec<_>>();

        let mut vertex_buffer_size = self.vertex_buffer.buffer().size();
        let mut index_buffer_size = self.index_buffer.buffer().size();
        for (_, v, i) in &new_geometry_data {
            vertex_buffer_size += v.len() as u64;
            index_buffer_size += i.len() as u64;
        }

        let vertex_buffer_len = vertex_buffer_size / Self::VERTEX_SIZE;
        let index_buffer_len = index_buffer_size / Self::INDEX_SIZE;
        let instance_buffer_len = (self.renderables.len() + renderables.len()) as u64;

        let (mut vertex_buffer, mut index_buffer, mut instance_buffer) = Self::init_buffers(
            vertex_buffer_len,
            index_buffer_len,
            instance_buffer_len,
            device,
        );

        let mut ordered_geometries =
            new_geometry_data
                .into_iter()
                .map(
                    |(geometry_key, vertex_bytes, index_bytes)| BufferedGeometry {
                        geometry_key,
                        vertices: CopyBytes::Raw(vertex_bytes),
                        indices: CopyBytes::Raw(index_bytes),
                    },
                )
                .chain(self.geometries.iter().map(|(geometry_key, geometry_data)| {
                    BufferedGeometry {
                        geometry_key: geometry_key.clone(),
                        vertices: CopyBytes::Src {
                            start: geometry_data.vertices_start,
                            size: Self::VERTEX_SIZE * geometry_data.vertices_len,
                        },
                        indices: CopyBytes::Src {
                            start: geometry_data.indices_start,
                            size: Self::INDEX_SIZE * geometry_data.indices_len,
                        },
                    }
                }))
                .collect::<Vec<_>>();
        ordered_geometries.sort_by_key(|g| g.geometry_key.clone());

        self.geometries.clear();
        let mut new_vertex_slices = vec![];
        let mut new_index_slices = vec![];
        let mut vertex_buffer_start = 0;
        let mut index_buffer_start = 0;

        for geometry in ordered_geometries.into_iter() {
            match (geometry.vertices, geometry.indices) {
                (CopyBytes::Raw(vertices), CopyBytes::Raw(indices)) => {
                    let vertices_len = vertices.len() as u64;
                    let indices_len = indices.len() as u64;

                    new_vertex_slices.push(BufferSlice::new(vertex_buffer_start, vertices));
                    new_index_slices.push(BufferSlice::new(index_buffer_start, indices));

                    self.geometries.insert(
                        geometry.geometry_key.clone(),
                        GeometryData {
                            vertices_start: vertex_buffer_start,
                            vertices_len,
                            indices_start: index_buffer_start,
                            indices_len,
                        },
                    );
                    vertex_buffer_start += vertices_len;
                    index_buffer_start += indices_len;
                }
                (
                    CopyBytes::Src {
                        start: vertex_start,
                        size: vertex_size,
                    },
                    CopyBytes::Src {
                        start: index_start,
                        size: index_size,
                    },
                ) => {
                    encoder.copy_buffer_to_buffer(
                        self.vertex_buffer.buffer(),
                        vertex_start,
                        vertex_buffer.buffer(),
                        vertex_buffer_start,
                        vertex_size,
                    );
                    encoder.copy_buffer_to_buffer(
                        self.index_buffer.buffer(),
                        index_start,
                        index_buffer.buffer(),
                        index_buffer_start,
                        index_size,
                    );

                    self.geometries.insert(
                        geometry.geometry_key.clone(),
                        GeometryData {
                            vertices_start: vertex_buffer_start,
                            vertices_len: vertex_size / Self::VERTEX_SIZE,
                            indices_start: index_buffer_start,
                            indices_len: index_size / Self::INDEX_SIZE,
                        },
                    );
                    vertex_buffer_start += vertex_size;
                    index_buffer_start += index_size;
                }
                _ => panic!(
                    "Expected both vertices and indices in BufferedGeometry to be either CopyBytes::Raw or CopyBytes::Src, got mix of both"
                ),
            }
        }

        vertex_buffer.write_slices(new_vertex_slices);
        index_buffer.write_slices(new_index_slices);

        let mut ordered_instances = new_instance_data
            .into_iter()
            .map(|(render_id, geometry_key, bytes)| BufferedInstance {
                render_id,
                geometry_key,
                bytes: CopyBytes::Raw(bytes),
            })
            .chain(
                self.renderables
                    .iter()
                    .map(|(render_id, data)| BufferedInstance {
                        render_id: *render_id,
                        geometry_key: data.geometry_key.clone(),
                        bytes: CopyBytes::Src {
                            start: Self::INSTANCE_SIZE * data.instance_index,
                            size: Self::INSTANCE_SIZE,
                        },
                    }),
            )
            .collect::<Vec<_>>();
        ordered_instances.sort_by_key(|instance| instance.render_id);

        self.renderables.clear();
        let mut instance_slices = vec![];

        for (dest_index, instance) in ordered_instances.into_iter().enumerate() {
            self.renderables.insert(
                instance.render_id,
                RenderableData {
                    geometry_key: instance.geometry_key,
                    instance_index: dest_index as u64,
                },
            );
            match instance.bytes {
                CopyBytes::Raw(bytes) => {
                    instance_slices.push(BufferSlice::new(
                        Self::INSTANCE_SIZE * dest_index as u64,
                        bytes,
                    ));
                }
                CopyBytes::Src { start, size } => {
                    encoder.copy_buffer_to_buffer(
                        self.instance_buffer.buffer(),
                        start,
                        instance_buffer.buffer(),
                        Self::INSTANCE_SIZE * dest_index as u64,
                        size,
                    );
                }
            }
        }

        instance_buffer.write_slices(instance_slices);

        self.vertex_buffer = vertex_buffer;
        self.index_buffer = index_buffer;
        self.instance_buffer = instance_buffer;
    }

    pub fn untrack(&mut self, render_ids: &[u64]) {
        for id in render_ids {
            self.renderables.remove(id);
        }
    }

    pub fn update(&mut self, renderables: &mut [&mut R], input: &mut RendererUpdateInput) {
        let mut geometry_bytes = HashMap::<(), ()>::with_capacity(renderables.len());
        let mut instance_bytes = HashMap::with_capacity(renderables.len());
        for renderable in renderables {
            let render_key = renderable.render_id();
            let geometry_key = renderable.geometry_key();
            let dirty_bytes = renderable.dirty_bytes();

            if !dirty_bytes.instance.is_empty() {
                instance_bytes.insert(render_key, dirty_bytes.instance);
            }
        }

        let mut ordered_instance_bytes = instance_bytes.into_iter().collect::<Vec<_>>();
        ordered_instance_bytes.sort_by_key(|(render_id, _)| *render_id);
    }

    pub fn render(
        &self,
        camera_bind_group: &BindGroup,
        atlas_bind_group: &BindGroup,
        render_pass: RenderPass<'_>,
        pipelines: &mut PipelinePool,
    ) {
        if self.renderables.is_empty() {
            return;
        }

        RenderPassDrawer::new()
            .bind_group(camera_bind_group)
            .bind_group(atlas_bind_group)
            .vertex_buffer(self.vertex_buffer.buffer())
            .vertex_buffer(self.instance_buffer.buffer())
            .index_buffer(
                self.index_buffer.buffer(),
                (self.index_buffer.buffer().size() / Self::INDEX_SIZE) as u32,
            )
            .instance_range(0..self.renderables.len() as u32)
            .draw(render_pass, pipelines.tilemap());
    }
}
