use wgpu::{
    Color, CommandEncoder, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView,
};

pub struct RenderPassFactory<'a> {
    texture_view: &'a TextureView,
    encoder: &'a mut CommandEncoder,
}

impl<'a> RenderPassFactory<'a> {
    pub fn new(texture_view: &'a TextureView, encoder: &'a mut CommandEncoder) -> Self {
        Self {
            texture_view,
            encoder,
        }
    }

    pub fn start(&'a mut self) -> RenderPass<'a> {
        self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.texture_view,
                resolve_target: None,
                depth_slice: None,
                ops: Operations {
                    store: StoreOp::Store,
                    //load: LoadOp::Load,
                    load: LoadOp::Clear(Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.9,
                        a: 1.0,
                    }),
                },
            })],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: None,
        })
    }

    pub fn secondary(&'a mut self) -> RenderPass<'a> {
        self.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.texture_view,
                resolve_target: None,
                depth_slice: None,
                ops: Operations {
                    store: StoreOp::Store,
                    load: LoadOp::Load,
                },
            })],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: None,
        })
    }
}
