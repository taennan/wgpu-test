use crate::{app::AppState, systems::AppSystem};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct AppCloser;

impl AppSystem for AppCloser {
    fn handle_event<'a>(
        &self,
        event: &'a WindowEvent,
        event_loop: &'a winit::event_loop::ActiveEventLoop,
        _app_state: &'a mut AppState,
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

    #[allow(unused)]
    fn run(&self, ctx: &mut AppState) {
        // NOOP
    }
}
