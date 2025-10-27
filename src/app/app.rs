use crate::{app::AppState, systems::*};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    window::{Theme, Window, WindowId},
};

#[derive(Default)]
pub struct App {
    app_state: Option<AppState>,
    systems_pool: AppSystemPool,
}

impl App {
    pub fn new() -> Self {
        let mut systems_pool = AppSystemPool::default();
        systems_pool.add(KeyMapper);
        systems_pool.add(AppCloser);
        systems_pool.add(CameraMover);
        //systems_pool.add(TextureToggler);
        systems_pool.add(WindowRedrawer);

        Self {
            app_state: None,
            systems_pool,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // NOTE: Don't know if this is necessary, just putting it here in case re-creating app state causes problems
        if self.app_state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("WGPU Test")
            .with_theme(Some(Theme::Dark));

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to get window"),
        );

        let state = AppState::try_new("start", window).expect("Failed to init AppState");
        self.app_state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(app_state) = &mut self.app_state {
            self.systems_pool
                .handle_event(&event, event_loop, app_state);
        }
    }
}
