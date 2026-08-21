use std::ops::Range;
use wgpu::{BindGroup, Buffer, BufferAddress, IndexFormat, RenderPass, RenderPipeline};

#[derive(Default)]
pub struct RenderPassDrawer<'a> {
    bind_groups: Vec<&'a BindGroup>,
    vertex_buffers: Vec<RenderPassVertices<'a>>,
    indices: Option<RenderPassIndices<'a>>,
    instance_range: Option<Range<u32>>,
}

enum RenderPassIndices<'a> {
    Range(Range<u32>),
    Buffer { buffer: &'a Buffer, length: u32 },
}

struct RenderPassVertices<'a> {
    buffer: &'a Buffer,
    range: Option<Range<BufferAddress>>,
}

impl<'a> RenderPassDrawer<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_groups(mut self, bind_groups: &'a [&'a BindGroup]) -> Self {
        for bind_group in bind_groups {
            self.bind_groups.push(bind_group);
        }
        self
    }

    pub fn vertex_buffer(mut self, buffer: &'a Buffer) -> Self {
        self.vertex_buffers.push(RenderPassVertices {
            buffer,
            range: None,
        });
        self
    }

    pub fn vertex_buffer_slice(mut self, buffer: &'a Buffer, range: Range<BufferAddress>) -> Self {
        self.vertex_buffers.push(RenderPassVertices {
            buffer,
            range: Some(range),
        });
        self
    }

    pub fn index_buffer(mut self, buffer: &'a Buffer, total_indices: u32) -> Self {
        self.indices = Some(RenderPassIndices::Buffer {
            buffer,
            length: total_indices,
        });
        self
    }

    pub fn index_range(mut self, range: Range<u32>) -> Self {
        self.indices = Some(RenderPassIndices::Range(range));
        self
    }

    pub fn instance_range(mut self, range: Range<u32>) -> Self {
        self.instance_range = Some(range);
        self
    }

    pub fn draw(&self, mut render_pass: RenderPass<'_>, render_pipeline: &RenderPipeline) {
        render_pass.set_pipeline(render_pipeline);
        for (index, bind_group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(index as u32, *bind_group, &[]);
        }

        for (index, vertex_buffer) in self.vertex_buffers.iter().enumerate() {
            let buffer = match &vertex_buffer.range {
                Some(range) => vertex_buffer.buffer.slice(range.clone()),
                _ => vertex_buffer.buffer.slice(..),
            };
            render_pass.set_vertex_buffer(index as u32, buffer);
        }

        let instance_range = self.instance_range.clone().unwrap_or(0..1);

        match &self.indices {
            Some(RenderPassIndices::Range(range)) => {
                render_pass.draw(range.clone(), instance_range);
            }
            Some(RenderPassIndices::Buffer { buffer, length }) => {
                render_pass.set_index_buffer(buffer.slice(..), IndexFormat::Uint32);
                render_pass.draw_indexed(0..*length, 0, instance_range);
            }
            _ => {
                log::error!("Either an index_buffer or index_range was not passed to RenderPass")
            }
        }
    }
}
