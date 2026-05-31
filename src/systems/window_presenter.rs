use wgpu::CurrentSurfaceTexture;

use crate::{app::RootState, systems::AppSystem};

#[derive(Debug)]
pub struct WindowPresenter;

impl AppSystem for WindowPresenter {
    fn run(&self, state: &mut RootState) {
        if !state.graphics.is_surface_configured {
            return;
        }

        log::debug!("Will present");
        let current_surface_result = state.graphics.surface.get_current_texture();
        match current_surface_result {
            CurrentSurfaceTexture::Success(surface) => surface.present(),
            CurrentSurfaceTexture::Suboptimal(surface) => {
                surface.present();
                log::warn!("Sub optimal texture received. Try calling Surface::configure again");
            }
            CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Timeout => {}
            CurrentSurfaceTexture::Lost => {
                log::warn!("Surface texture lost");
            }
            CurrentSurfaceTexture::Outdated => {
                log::warn!("Surface texture outdated. Need to call Surface::configure again");
            }
            CurrentSurfaceTexture::Validation => {
                log::warn!("Surface texture validation failed");
            }
        };
    }
}
