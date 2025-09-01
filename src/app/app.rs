use crate::{
    app::AppState,
    systems::{
        AppCloser, AppSystem, AppSystemManager, CameraMover, TextureToggler, WindowEventContext,
        WindowRedrawer,
    },
};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    window::{Theme, Window, WindowId},
};

#[derive(Default)]
pub struct App {
    state: Option<AppState>,
    systems: AppSystemManager,
}

impl App {
    pub fn new() -> Self {
        let mut systems = AppSystemManager::default();
        systems.add(AppCloser::new());
        systems.add(CameraMover::default());
        systems.add(TextureToggler::new());
        systems.add(WindowRedrawer::new());

        Self {
            state: None,
            systems,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("WGPU Test")
            .with_theme(Some(Theme::Dark));

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to get window"),
        );

        let state = futures::executor::block_on(AppState::try_new(window))
            .expect("Failed to init AppState");
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = &mut self.state {
            let mut event_context = WindowEventContext {
                event,
                event_loop: event_loop,
                app_state: state,
            };
            self.systems.handle_window_event(&mut event_context);
        }
        /*

        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        let mut event_context = WindowEventContext {
            event,
            event_loop: event_loop,
            app_state: state,
        };
        self.systems.handle_window_event(&mut event_context);
        */
    }
}
