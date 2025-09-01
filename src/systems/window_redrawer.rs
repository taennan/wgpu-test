use crate::{
    error::*,
    graphics::RenderPass,
    systems::{AppSystem, WindowEventContext},
};
use winit::{dpi::PhysicalSize, event::WindowEvent};

pub struct WindowRedrawer;

impl WindowRedrawer {
    pub fn new() -> Self {
        Self
    }

    fn resize(&self, ctx: &mut WindowEventContext, size: &PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            ctx.app_state.surface_config.width = size.width;
            ctx.app_state.surface_config.height = size.height;
            ctx.app_state
                .surface
                .configure(&ctx.app_state.device, &ctx.app_state.surface_config);
            ctx.app_state.is_surface_configured = true;
        }
    }

    fn render_or_resize_on_error(&self, ctx: &mut WindowEventContext) {
        match self.render(ctx) {
            Err(Error::Texture(TextureError::Lost) | Error::Texture(TextureError::Outdated)) => {
                self.resize(ctx, &ctx.app_state.window.inner_size());
            }
            Err(error) => log::error!("Failed to redraw window {:?}", error),
            _ => {}
        }
    }

    fn render(&self, ctx: &mut WindowEventContext) -> Result<()> {
        let app_state = &mut ctx.app_state;

        app_state.window.request_redraw();

        if !app_state.is_surface_configured {
            return Ok(());
        }

        let texture_output =
            app_state
                .surface
                .get_current_texture()
                .map_err(|error| match error {
                    wgpu::SurfaceError::Lost => Error::Texture(TextureError::Lost),
                    wgpu::SurfaceError::Outdated => Error::Texture(TextureError::Outdated),
                    _ => Error::Texture(TextureError::Other),
                })?;
        let texture_view = texture_output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            app_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        let _ = app_state.device.poll(wgpu::PollType::Wait);

        app_state.camera_manager.update_main_buffer(&mut encoder);

        let plain_shape_render_pass = RenderPass::builder()
            .bind_group(app_state.simple_bind_group.group())
            .bind_group(app_state.camera_manager.bind_group())
            .index_buffer(&app_state.index_buffer, app_state.num_indices)
            .vertex_buffer(&app_state.vertex_buffer)
            .build(
                "Plain Shape",
                &texture_view,
                app_state.render_pipeline.pipeline(),
            );

        plain_shape_render_pass.begin(&mut encoder);

        app_state.queue.submit(std::iter::once(encoder.finish()));
        texture_output.present();

        Ok(())
    }
}

impl AppSystem for WindowRedrawer {
    fn handle_window_event(&mut self, ctx: &mut WindowEventContext) {
        match ctx.event {
            WindowEvent::Resized(size) => self.resize(ctx, &size),
            WindowEvent::RedrawRequested => self.render_or_resize_on_error(ctx),
            _ => {}
        }
    }
}
