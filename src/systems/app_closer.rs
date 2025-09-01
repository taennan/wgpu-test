use crate::systems::{AppSystem, WindowEventContext};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct AppCloser;

impl AppCloser {
    pub fn new() -> Self {
        Self
    }
}

impl AppSystem for AppCloser {
    fn handle_window_event(&mut self, ctx: &mut WindowEventContext) {
        match &ctx.event {
            WindowEvent::CloseRequested => ctx.event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(KeyCode::Escape) => ctx.event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }
}
