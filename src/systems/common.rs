use crate::app::AppState;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

pub trait AppSystem {
    fn handle_window_event(&mut self, ctx: &mut WindowEventContext<'_>);
}

pub struct WindowEventContext<'a> {
    pub event: WindowEvent,
    pub event_loop: &'a ActiveEventLoop,
    pub app_state: &'a mut AppState,
}

#[derive(Default)]
pub struct AppSystemManager {
    systems: Vec<Box<dyn AppSystem>>,
}

impl AppSystemManager {
    pub fn add<T: AppSystem + 'static>(&mut self, system: T) {
        self.systems.push(Box::new(system));
    }
}

impl AppSystem for AppSystemManager {
    fn handle_window_event(&mut self, ctx: &mut WindowEventContext<'_>) {
        for entity in self.systems.iter_mut() {
            entity.handle_window_event(ctx);
        }
    }
}
