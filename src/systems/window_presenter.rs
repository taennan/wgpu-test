use crate::{app::RootState, systems::AppSystem};

pub struct WindowPresenter;

impl AppSystem for WindowPresenter {
    fn run(&self, state: &mut RootState) {
        if !state.graphics.is_surface_configured {
            return;
        }

        let texture_output = state
            .graphics
            .surface
            .get_current_texture()
            /*
            .map_err(|error| match error {
                wgpu::SurfaceError::Lost => Error::Texture(TextureError::Lost),
                wgpu::SurfaceError::Outdated => Error::Texture(TextureError::Outdated),
                _ => Error::Texture(TextureError::Other),
            })
             */
            .expect("Failed to get presented texture");
        texture_output.present();
    }
}
