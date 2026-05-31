use crate::{
    app::RootState,
    systems::{AppSystem, AppSystemContext},
};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug)]
pub struct TextureToggler;

impl AppSystem for TextureToggler {
    fn run(&self, ctx: &mut AppSystemContext) {
        match &*ctx.event {
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(KeyCode::Digit1) => {
                    if event.state == ElementState::Released {
                        self.toggle_texture(&mut ctx.root_state);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl TextureToggler {
    fn toggle_texture(&self, root_state: &mut RootState) {
        let next_texture_type = root_state.current_texture_type.other();
        let next_texture_key = &next_texture_type.to_str();

        let _ = root_state.texture_pool.load(next_texture_key);

        let new_texture = match root_state.texture_pool.get(next_texture_key) {
            Some(texture) => texture,
            _ => {
                log::error!("Failed to load texture");
                return;
            }
        };
        root_state
            .simple_bind_group
            .set_texture(&new_texture, &root_state.device);

        root_state
            .texture_pool
            .unload(&root_state.current_texture_type.to_str());
        root_state.current_texture_type = next_texture_type;
        root_state.window.request_redraw();
    }
}
