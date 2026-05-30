use std::{collections::HashSet, sync::Arc};
use winit::{keyboard::KeyCode, window::Window};

pub struct AppState {
    pub keys_pressed: HashSet<KeyCode>,
    pub window: Arc<Window>,
}

impl AppState {
    pub fn new(window: Arc<Window>) -> Self {
        Self {
            window,
            keys_pressed: HashSet::new(),
        }
    }
}
