use crate::{app::AppState, systems::AppSystem};
use winit::{
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
};

pub struct KeyMapper;

impl AppSystem for KeyMapper {
    fn handle_event<'a>(
        &self,
        event: &'a WindowEvent,
        _event_loop: &'a ActiveEventLoop,
        app_state: &'a mut AppState,
    ) {
        match &event {
            WindowEvent::KeyboardInput { event, .. } => match (event.physical_key, event.state) {
                (PhysicalKey::Code(key_code), ElementState::Pressed) => {
                    app_state.keys_pressed.insert(key_code);
                }
                (PhysicalKey::Code(key_code), ElementState::Released) => {
                    app_state.keys_pressed.remove(&key_code);
                }
                _ => {}
            },
            _ => {}
        }
    }

    #[allow(unused)]
    fn run(&self, ctx: &mut AppState) {
        // NOOP
    }
}
