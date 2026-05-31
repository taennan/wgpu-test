use crate::{app::RootState, systems::AppSystem};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug)]
pub struct AppCloser;

impl AppSystem for AppCloser {
    fn handle_event<'a>(
        &self,
        event: &'a WindowEvent,
        event_loop: &'a winit::event_loop::ActiveEventLoop,
        _root_state: &'a mut RootState,
    ) {
        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }
}
