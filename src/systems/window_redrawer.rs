use crate::{
    app::RootState,
    error::*,
    graphics::{RendererUpdateInput, pipeline::RenderPassFactory},
    systems::AppSystem,
};
use glam::UVec2;
use std::iter;
use wgpu::{CurrentSurfaceTexture, PollType};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

#[derive(Debug)]
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
        let size = UVec2::new(size.width, size.height);
        if size.x > 0 && size.y > 0 {
            state.graphics.surface_config.width = size.x;
            state.graphics.surface_config.height = size.y;
            state
                .graphics
                .surface
                .configure(&state.graphics.device, &state.graphics.surface_config);
            state.game.camera.update_on_screen_resize(size);
            state.graphics.screen_size.set_value(size);
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

        let current_surface_result = state.graphics.surface.get_current_texture();
        let current_texture = match current_surface_result {
            CurrentSurfaceTexture::Success(surface) => surface,
            CurrentSurfaceTexture::Suboptimal(surface) => {
                log::warn!("Sub optimal texture received. Try calling Surface::configure again");
                surface
            }
            CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Timeout => {
                return Ok(());
            }
            CurrentSurfaceTexture::Lost => {
                log::warn!("Surface texture lost");
                return Ok(());
            }
            CurrentSurfaceTexture::Outdated => {
                log::warn!("Surface texture outdated. Need to call Surface::configure again");
                return Ok(());
            }
            CurrentSurfaceTexture::Validation => {
                log::warn!("Surface texture validation failed");
                return Ok(());
            }
        };

        let texture_view = current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("CurrentSurfaceTextureView"),
                ..Default::default()
            });

        let mut encoder =
            state
                .graphics
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        let mut renderer_update_input = RendererUpdateInput {
            texture_atlas: &state.graphics.texture_atlas,
            geometry: &state.graphics.geometry_pool,
            device: &state.graphics.device,
        };

        state.graphics.camera_renderer.update(&state.game.camera);

        // Must wrap in blocks so that we can mutably borrow encoder later
        if !state.graphics.pipelines.is_mesh_disabled {
            state
                .graphics
                .mesh_renderer
                .update(&state.game.meshes, &mut renderer_update_input);
            let mut render_pass_factory = RenderPassFactory::new(&texture_view, &mut encoder);
            state.graphics.mesh_renderer.render(
                state.graphics.camera_renderer.bind_group(),
                state.graphics.texture_atlas.bind_group(),
                render_pass_factory.start(),
                &mut state.graphics.pipelines,
            );
        }
        if !state.graphics.pipelines.is_sprite_disabled {
            state
                .graphics
                .sprite_renderer
                .update(&state.game.sprites, &mut renderer_update_input);
            let mut render_pass_factory = RenderPassFactory::new(&texture_view, &mut encoder);
            state.graphics.sprite_renderer.render(
                state.graphics.texture_atlas.bind_group(),
                state.graphics.screen_size.bind_group(),
                render_pass_factory.secondary(),
                &mut state.graphics.pipelines,
            );
        }

        state
            .graphics
            .device
            .poll(PollType::wait_indefinitely())
            .expect("Failed to poll device");

        /*
        state.graphics.surface_texture_view = Some(texture_view);
        state.graphics.command_buffers.push(encoder.finish());
         */

        state.graphics.queue.submit(iter::once(encoder.finish()));
        state.app.window.pre_present_notify();
        state.graphics.queue.present(current_texture);

        Ok(())
    }
}
