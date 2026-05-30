use super::AppSystem;
use crate::app::RootState;
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
        root_state: &'a mut RootState,
    ) {
        for system in self.systems.iter() {
            if system.can_run(root_state) {
                system.handle_event(&event, event_loop, root_state);
                system.run(root_state);
            }
        }
    }
}
