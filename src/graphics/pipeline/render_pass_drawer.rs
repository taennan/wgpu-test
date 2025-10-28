use std::ops::Range;
use wgpu::{BindGroup, Buffer, BufferAddress, IndexFormat, RenderPass, RenderPipeline};

#[derive(Default)]
pub struct RenderPassDrawer<'a> {
    bind_groups: Vec<&'a BindGroup>,
    vertex_buffers: Vec<RenderPassVertices<'a>>,
    indices: Option<RenderPassIndices<'a>>,
    index_range: Option<Range<u32>>,
    instance_range: Option<Range<u32>>,
}

pub struct RenderPassDrawInput<'a> {
    pub render_pass: &'a mut RenderPass<'a>,
    pub render_pipeline: &'a RenderPipeline,
}

struct RenderPassIndices<'a> {
    buffer: &'a Buffer,
    length: u32,
}

struct RenderPassVertices<'a> {
    buffer: &'a Buffer,
    range: Option<Range<BufferAddress>>,
}

impl<'a> RenderPassDrawer<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_group(mut self, bind_group: &'a BindGroup) -> Self {
        self.bind_groups.push(bind_group);
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
        self.indices = Some(RenderPassIndices {
            buffer,
            length: total_indices,
        });
        self
    }

    pub fn index_range(mut self, range: Range<u32>) -> Self {
        self.index_range = Some(range);
        self
    }

    pub fn instance_range(mut self, range: Range<u32>) -> Self {
        self.instance_range = Some(range);
        self
    }

    pub fn draw(&self, render_pass: &mut RenderPass<'_>, render_pipeline: &RenderPipeline) {
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

        if let Some(indices) = &self.indices {
            render_pass.set_index_buffer(indices.buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.length, 0, instance_range);
        } else if let Some(index_range) = &self.index_range {
            render_pass.draw(index_range.clone(), instance_range);
        } else {
            log::error!("One of indices or index_range was not passed to RenderPass")
        }
    }
}
