use crate::{
    app::RootState, error::*, graphics::RendererUpdateInput, graphics::pipeline::RenderPassFactory,
    systems::AppSystem,
};
use wgpu::PollType;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

pub struct WindowRedrawer;

impl AppSystem for WindowRedrawer {
    fn handle_event<'a>(
        &self,
        event: &'a WindowEvent,
        _event_loop: &'a ActiveEventLoop,
        state: &'a mut RootState,
    ) {
        match &event {
            // Removed the size arg from the resize method call just in case the current size can be found via state.window.inner_size()
            WindowEvent::Resized(_) => self.resize(state),
            WindowEvent::RedrawRequested => self.render_or_resize_on_error(state),
            _ => {}
        }
    }
}

impl WindowRedrawer {
    fn resize(&self, state: &mut RootState) {
        let size = state.app.window.inner_size();
        if size.width > 0 && size.height > 0 {
            state.graphics.surface_config.width = size.width;
            state.graphics.surface_config.height = size.height;
            state
                .graphics
                .surface
                .configure(&state.graphics.device, &state.graphics.surface_config);
            state.graphics.is_surface_configured = true;
        }
    }

    fn render_or_resize_on_error(&self, state: &mut RootState) {
        match self.render(state) {
            Err(Error::Texture(TextureError::Lost) | Error::Texture(TextureError::Outdated)) => {
                self.resize(state);
            }
            Err(error) => log::error!("Failed to redraw window {:?}", error),
            _ => {}
        }
    }

    fn render(&self, state: &mut RootState) -> Result<()> {
        state.app.window.request_redraw();

        if !state.graphics.is_surface_configured {
            return Ok(());
        }

        let texture_output = state
            .graphics
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
            state
                .graphics
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        let mut renderer_update_input = RendererUpdateInput {
            texture_atlas: &mut state.graphics.texture_atlas,
            texture_buffers: &mut state.graphics.texture_buffers,
            geometry: &mut state.graphics.geometry_pool,
            pipelines: &mut state.graphics.pipeline_pool,
            device: &state.graphics.device,
            encoder: &mut encoder,
        };

        state.graphics.camera_renderer.update(&state.game.camera);
        state
            .graphics
            .mesh_renderer
            .update(&state.game.meshes, &mut renderer_update_input);
        /*

        state
            .graphics
            .sprite_renderer
            .update(&state.game.sprites, &mut renderer_update_input);
         */

        {
            // Must wrap in block so that we can mutably borrow encoder later
            let mut render_pass_factory = RenderPassFactory::new(&texture_view, &mut encoder);
            state.graphics.mesh_renderer.render(
                state.graphics.camera_renderer.bind_group(),
                render_pass_factory.start(),
                &mut state.graphics.pipeline_pool,
            );
        }
        /*

        {
            let mut render_pass_factory = RenderPassFactory::new(&texture_view, &mut encoder);
            state.graphics.sprite_renderer.render(
                state.graphics.texture_atlas.bind_group_unchecked(),
                render_pass_factory.start(),
                &mut state.graphics.pipeline_pool,
            );
        }
         */

        state
            .graphics
            .device
            .poll(PollType::wait_indefinitely())
            .expect("Failed to poll device");

        state.graphics.command_buffers.push(encoder.finish());
        /*
        state
            .graphics
            .queue
            .submit(std::iter::once(encoder.finish()));
        texture_output.present();
         */

        Ok(())
    }
}
