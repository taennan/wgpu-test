use crate::{
    app::AppState,
    systems::{AppSystem, AppSystemContext},
};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct TextureToggler;

impl AppSystem for TextureToggler {
    fn run(&self, ctx: &mut AppSystemContext) {
        match &*ctx.event {
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(KeyCode::Digit1) => {
                    if event.state == ElementState::Released {
                        self.toggle_texture(&mut ctx.app_state);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl TextureToggler {
    fn toggle_texture(&self, app_state: &mut AppState) {
        let next_texture_type = app_state.current_texture_type.other();
        let next_texture_key = &next_texture_type.to_str();

        let _ = app_state.texture_pool.load(next_texture_key);

        let new_texture = match app_state.texture_pool.get(next_texture_key) {
            Some(texture) => texture,
            _ => {
                log::error!("Failed to load texture");
                return;
            }
        };
        app_state
            .simple_bind_group
            .set_texture(&new_texture, &app_state.device);

        app_state
            .texture_pool
            .unload(&app_state.current_texture_type.to_str());
        app_state.current_texture_type = next_texture_type;
        app_state.window.request_redraw();
    }
}
