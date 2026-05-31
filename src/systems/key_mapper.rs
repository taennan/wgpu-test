use crate::{app::RootState, systems::AppSystem};
use winit::{
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
};

#[derive(Debug)]
pub struct KeyMapper;

impl AppSystem for KeyMapper {
    fn handle_event<'a>(
        &self,
        event: &'a WindowEvent,
        _event_loop: &'a ActiveEventLoop,
        state: &'a mut RootState,
    ) {
        match &event {
            WindowEvent::KeyboardInput { event, .. } => match (event.physical_key, event.state) {
                (PhysicalKey::Code(key_code), ElementState::Pressed) => {
                    state.app.keys_pressed.insert(key_code);
                }
                (PhysicalKey::Code(key_code), ElementState::Released) => {
                    state.app.keys_pressed.remove(&key_code);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
