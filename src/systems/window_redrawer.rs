use crate::{
    app::{AppState, app},
    error::*,
    scene::{RendererRenderInput, RendererUpdateInput},
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
        app_state: &'a mut AppState,
    ) {
        match &event {
            // NOTE: Removed the size arg from the resize method call just in case the current size can be found via app_state.window.inner_size()
            WindowEvent::Resized(_) => self.resize(app_state),
            WindowEvent::RedrawRequested => self.render_or_resize_on_error(app_state),
            _ => {}
        }
    }

    #[allow(unused)]
    fn run(&self, state: &mut AppState) {}
}

impl WindowRedrawer {
    fn resize(&self, app_state: &mut AppState) {
        let size = app_state.window.inner_size();
        if size.width > 0 && size.height > 0 {
            app_state.surface_config.width = size.width;
            app_state.surface_config.height = size.height;
            app_state
                .surface
                .configure(&app_state.device, &app_state.surface_config);
            app_state.is_surface_configured = true;
        }
    }

    fn render_or_resize_on_error(&self, app_state: &mut AppState) {
        match self.render(app_state) {
            Err(Error::Texture(TextureError::Lost) | Error::Texture(TextureError::Outdated)) => {
                self.resize(app_state);
            }
            Err(error) => log::error!("Failed to redraw window {:?}", error),
            _ => {}
        }
    }

    fn render(&self, app_state: &mut AppState) -> Result<()> {
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

        app_state.scene_renderer.update(
            &app_state.scene,
            &mut RendererUpdateInput {
                texture_atlas: &mut app_state.texture_atlas,
                texture_buffers: &mut app_state.texture_buffers,
                textures: &mut app_state.texture_pool,
                geometry: &mut app_state.geometry_pool,
                pipelines: &mut app_state.pipeline_pool,
                device: &app_state.device,
                encoder: &mut encoder,
            },
        );

        app_state.scene_renderer.render(&mut RendererRenderInput {
            texture_view: &texture_view,
            pipelines: &mut app_state.pipeline_pool,
            encoder: &mut encoder,
        });

        app_state
            .device
            .poll(PollType::wait_indefinitely())
            .expect("Failed to poll device");

        app_state.queue.submit(std::iter::once(encoder.finish()));
        texture_output.present();

        Ok(())
    }
}
