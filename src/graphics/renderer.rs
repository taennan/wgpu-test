use crate::graphics::buffer::{BufferSlice, MutBuffer};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    marker::PhantomData,
    mem,
    path::PathBuf,
    sync::atomic::AtomicU64,
};
use wgpu::{BindGroup, BufferUsages, CommandEncoder, Device, RenderPass, RenderPipeline};

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
    pub geometry_key: PathBuf,
    pub vertices: CopyBytes,
    pub indices: CopyBytes,
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

impl CopyBytes {
    pub fn size(&self) -> u64 {
        match &self {
            Self::Raw(bytes) => bytes.len() as u64,
            Self::Src { size, .. } => *size,
        }
    }
}

pub static RENDERABLE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub trait Renderable {
    fn render_id(&self) -> u64;
    fn geometry_key(&self) -> PathBuf;
    fn vertex_bytes(&self) -> (Box<[u8]>, Box<[u8]>);
    fn instance_bytes(&self) -> Box<[u8]>;
    fn dirty_instance_bytes(&mut self) -> Box<[u8]>;
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

        log::debug!(
            "Initialising Renderer MutBuffers with lengths and sizes, vertex=({}, {}), index=({}, {}), instance=({}, {})",
            vertex_buffer_len,
            vertex_buffer.buffer().size(),
            index_buffer_len,
            index_buffer.buffer().size(),
            instance_buffer_len,
            instance_buffer.buffer().size(),
        );

        (vertex_buffer, index_buffer, instance_buffer)
    }

    pub fn track(&mut self, renderables: &[&R], device: &Device, encoder: &mut CommandEncoder) {
        let mut new_geometry_keys = HashSet::new();
        let new_geometry_data = renderables
            .iter()
            .filter(|r| {
                let key = r.geometry_key();
                let is_already_done = new_geometry_keys.contains(&key);
                let is_old_geometry = self.geometries.contains_key(&key);

                if is_already_done || is_old_geometry {
                    false
                } else {
                    new_geometry_keys.insert(key);
                    true
                }
            })
            .map(|r| {
                let (vertex_bytes, index_bytes) = r.vertex_bytes();
                (r.geometry_key(), vertex_bytes, index_bytes)
            })
            .collect::<Vec<_>>();
        let new_instance_data = renderables
            .iter()
            .filter(|r| !self.renderables.contains_key(&r.render_id()))
            .map(|r| (r.render_id(), r.geometry_key(), r.instance_bytes()))
            .collect::<Vec<_>>();
        if new_geometry_data.is_empty() && new_instance_data.is_empty() {
            return;
        }

        log::debug!(
            "Got geometries new={} old={}",
            new_geometry_data.len(),
            self.geometries.len()
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
        ordered_geometries.sort_by(|a, b| a.geometry_key.cmp(&b.geometry_key));

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

        let vertex_buffer_len = ordered_geometries
            .iter()
            .map(|g| g.vertices.size())
            .sum::<u64>()
            / Self::VERTEX_SIZE;
        let index_buffer_len = ordered_geometries
            .iter()
            .map(|g| g.indices.size())
            .sum::<u64>()
            / Self::INDEX_SIZE;
        let instance_buffer_len = ordered_instances
            .iter()
            .map(|i| i.bytes.size())
            .sum::<u64>()
            / Self::INSTANCE_SIZE;
        let (mut vertex_buffer, mut index_buffer, mut instance_buffer) = Self::init_buffers(
            vertex_buffer_len,
            index_buffer_len,
            instance_buffer_len,
            device,
        );

        self.geometries.clear();
        self.renderables.clear();

        let mut new_vertex_slices = vec![];
        let mut new_index_slices = vec![];
        let mut instance_slices = vec![];
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
                    log::debug!(
                        "Copying vertices from buffer to buffer, source_start={} dest_start={} size={}",
                        vertex_start,
                        vertex_buffer_start,
                        vertex_size
                    );
                    encoder.copy_buffer_to_buffer(
                        self.vertex_buffer.buffer(),
                        vertex_start,
                        vertex_buffer.buffer(),
                        vertex_buffer_start,
                        vertex_size,
                    );

                    log::debug!(
                        "Copying indices from buffer to buffer, source_start={} dest_start={} size={}",
                        index_start,
                        index_buffer_start,
                        index_size
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

        log::debug!("Writing new vertices, indices and instance buffer slices");
        vertex_buffer.write_slices(new_vertex_slices);
        index_buffer.write_slices(new_index_slices);
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

    pub fn untrack_all(&mut self) {
        self.renderables.clear();
    }

    pub fn update(&mut self, renderables: &mut [R]) {
        let mut updatable_instance_slices = Vec::with_capacity(renderables.len());

        for renderable in renderables {
            let dirty_bytes = renderable.dirty_instance_bytes();
            let instance_start = self
                .renderables
                .get(&renderable.render_id())
                .map(|d| d.instance_index * Self::INSTANCE_SIZE);

            if !dirty_bytes.is_empty()
                && let Some(instance_start) = instance_start
            {
                log::debug!(
                    "updating instance bytes render_id={}, instance_start={}",
                    renderable.render_id(),
                    instance_start
                );
                updatable_instance_slices.push(BufferSlice::new(instance_start, dirty_bytes));
            }
        }

        self.instance_buffer.write_slices(updatable_instance_slices);
    }

    pub fn render(
        &self,
        bind_groups: &[&BindGroup],
        mut render_pass: RenderPass<'_>,
        pipeline: &RenderPipeline,
    ) {
        if self.renderables.is_empty() {
            return;
        }

        render_pass.set_pipeline(pipeline);
        for (index, bind_group) in bind_groups.iter().enumerate() {
            render_pass.set_bind_group(index as u32, *bind_group, &[]);
        }

        for (index, buffer) in [self.vertex_buffer.buffer(), self.instance_buffer.buffer()]
            .iter()
            .enumerate()
        {
            let buffer_slice = buffer.slice(..);
            render_pass.set_vertex_buffer(index as u32, buffer_slice);
        }

        let total_indices = self.index_buffer.buffer().size() / Self::INDEX_SIZE;

        render_pass.set_index_buffer(
            self.index_buffer.buffer().slice(..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..total_indices as u32, 0, 0..self.renderables.len() as u32);
    }
}
