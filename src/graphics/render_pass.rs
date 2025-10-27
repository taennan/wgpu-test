use std::ops::Range;
use wgpu::{BindGroup, Buffer, BufferAddress, CommandEncoder, RenderPipeline, TextureView};

pub struct RenderPass<'a> {
    name: &'static str,
    texture_view: &'a TextureView,
    render_pipeline: &'a RenderPipeline,
    bind_groups: Vec<&'a BindGroup>,
    vertex_buffers: Vec<RenderPassVertices<'a>>,
    indices: Option<RenderPassIndices<'a>>,
    index_range: Option<Range<u32>>,
    instance_range: Option<Range<u32>>,
}

#[derive(Default)]
pub struct RenderPassBuilder<'a> {
    bind_groups: Vec<&'a BindGroup>,
    vertex_buffers: Vec<RenderPassVertices<'a>>,
    indices: Option<RenderPassIndices<'a>>,
    index_range: Option<Range<u32>>,
    instance_range: Option<Range<u32>>,
}

struct RenderPassIndices<'a> {
    buffer: &'a Buffer,
    length: u32,
}

struct RenderPassVertices<'a> {
    buffer: &'a Buffer,
    range: Option<Range<BufferAddress>>,
}

impl<'a> RenderPassBuilder<'a> {
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

    pub fn build(
        self,
        name: &'static str,
        texture_view: &'a TextureView,
        render_pipeline: &'a RenderPipeline,
    ) -> RenderPass<'a> {
        RenderPass {
            name,
            texture_view,
            render_pipeline,
            bind_groups: self.bind_groups,
            vertex_buffers: self.vertex_buffers,
            instance_range: self.instance_range,
            indices: self.indices,
            index_range: self.index_range,
        }
    }
}

impl<'a> RenderPass<'a> {
    pub fn builder() -> RenderPassBuilder<'a> {
        RenderPassBuilder::default()
    }

    pub fn begin(&self, encoder: &mut CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("{} Render Pass", self.name)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.texture_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    store: wgpu::StoreOp::Store,
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.2,
                        g: 0.2,
                        b: 1.0,
                        a: 1.0,
                    }),
                },
            })],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        for (index, bind_group) in self.bind_groups.iter().enumerate() {
            //log::debug!("Setting bind group {} {:?}", index, bind_group);
            render_pass.set_bind_group(index as u32, *bind_group, &[]);
        }

        for (index, vertex_buffer) in self.vertex_buffers.iter().enumerate() {
            let buffer = match &vertex_buffer.range {
                Some(range) => vertex_buffer.buffer.slice(range.clone()),
                _ => {
                    //log::debug!("Setting vertex buffer {} {:?}", index, vertex_buffer.buffer);
                    vertex_buffer.buffer.slice(..)
                }
            };
            render_pass.set_vertex_buffer(index as u32, buffer);
        }

        let instance_range = self.instance_range.clone().unwrap_or(0..1);

        if let Some(indices) = &self.indices {
            render_pass.set_index_buffer(indices.buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.length, 0, instance_range);
        } else if let Some(index_range) = &self.index_range {
            //log::debug!("Drawing {:?} {:?}", index_range, instance_range);
            render_pass.draw(index_range.clone(), instance_range);
        } else {
            log::error!("One of indices or index_range was not passed to RenderPass")
        }
    }
}
