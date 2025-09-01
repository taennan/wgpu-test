use wgpu::{BindGroup, Buffer, CommandEncoder, RenderPipeline, TextureView};

struct RenderPassIndices<'a> {
    buffer: &'a Buffer,
    length: u32,
}

pub struct RenderPass<'a> {
    name: &'static str,
    texture_view: &'a TextureView,
    render_pipeline: &'a RenderPipeline,
    bind_groups: Vec<&'a BindGroup>,
    vertices: Option<&'a Buffer>,
    indices: Option<RenderPassIndices<'a>>,
}

#[derive(Default)]
pub struct RenderPassBuilder<'a> {
    bind_groups: Vec<&'a BindGroup>,
    vertices: Option<&'a Buffer>,
    indices: Option<RenderPassIndices<'a>>,
}

impl<'a> RenderPassBuilder<'a> {
    pub fn bind_group(mut self, bind_group: &'a BindGroup) -> Self {
        self.bind_groups.push(bind_group);
        self
    }

    pub fn vertex_buffer(mut self, buffer: &'a Buffer) -> Self {
        self.vertices = Some(buffer);
        self
    }

    pub fn index_buffer(mut self, buffer: &'a Buffer, total_indices: u32) -> Self {
        self.indices = Some(RenderPassIndices {
            buffer,
            length: total_indices,
        });
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
            vertices: self.vertices,
            indices: self.indices,
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
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 1.0,
                        a: 0.3,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        for (index, bind_group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(index as u32, *bind_group, &[]);
        }

        if let Some(vertices) = &self.vertices {
            render_pass.set_vertex_buffer(0, vertices.slice(..));
        }
        if let Some(indices) = &self.indices {
            render_pass.set_index_buffer(indices.buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..indices.length, 0, 0..1);
        }
    }
}
