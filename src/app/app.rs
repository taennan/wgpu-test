use crate::{app::RootState, systems::*};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    window::{Theme, Window, WindowId},
};

#[derive(Default)]
pub struct App {
    root_state: Option<RootState>,
    systems_pool: AppSystemPool,
}

impl App {
    pub fn new() -> Self {
        let mut systems_pool = AppSystemPool::default();
        systems_pool.add(KeyMapper);
        systems_pool.add(AppCloser);
        systems_pool.add(SceneLoader);
        systems_pool.add(CameraMover);
        //systems_pool.add(TextureToggler);
        systems_pool.add(WindowRedrawer);
        //systems_pool.add(CommandSubmitter);

        log::info!("Added systems");

        Self {
            root_state: None,
            systems_pool,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // NOTE: Don't know if this is necessary, just putting it here in case re-creating app state causes problems
        if self.root_state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title("WGPU Test")
            .with_theme(Some(Theme::Dark))
            .with_active(true);

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Failed to get window"),
        );

        log::debug!("Will init root state");
        let state = RootState::try_new("start", window).expect("Failed to init RootState");
        log::debug!("Did init root state");
        self.root_state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(root_state) = &mut self.root_state {
            self.systems_pool
                .handle_event(&event, event_loop, root_state);
        }
    }
}
