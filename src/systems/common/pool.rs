use super::AppSystem;
use crate::app::AppState;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

#[derive(Default)]
pub struct AppSystemPool {
    systems: Vec<Box<dyn AppSystem + 'static>>,
}

impl AppSystemPool {
    pub fn add(&mut self, system: impl AppSystem + 'static) {
        self.systems.push(Box::new(system));
    }

    pub fn handle_event<'a>(
        &mut self,
        event: &'a WindowEvent,
        event_loop: &'a ActiveEventLoop,
        app_state: &'a mut AppState,
    ) {
        for system in self.systems.iter() {
            if system.can_run(app_state) {
                system.handle_event(&event, event_loop, app_state);
                system.run(app_state);
            }
        }
    }
}
