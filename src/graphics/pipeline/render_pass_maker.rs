use wgpu::{
    Color, CommandEncoder, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView,
};

pub fn start<'a>(encoder: &'a mut CommandEncoder, texture_view: &'a TextureView) -> RenderPass<'a> {
    create_render_pass(
        "Start Render Pass",
        encoder,
        texture_view,
        LoadOp::Clear(Color {
            r: 0.3,
            g: 0.3,
            b: 0.9,
            a: 1.0,
        }),
    )
}

pub fn secondary<'a>(
    encoder: &'a mut CommandEncoder,
    texture_view: &'a TextureView,
) -> RenderPass<'a> {
    create_render_pass("Secondary Render Pass", encoder, texture_view, LoadOp::Load)
}

fn create_render_pass<'a>(
    label: &str,
    encoder: &'a mut CommandEncoder,
    texture_view: &'a TextureView,
    load_op: LoadOp<Color>,
) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: texture_view,
            resolve_target: None,
            depth_slice: None,
            ops: Operations {
                store: StoreOp::Store,
                load: load_op,
            },
        })],
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
        depth_stencil_attachment: None,
    })
}
