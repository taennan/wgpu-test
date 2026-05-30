use crate::{
    app::{AppState, GameState, GraphicsState},
    error::*,
};
use std::sync::Arc;
use winit::window::Window;

pub struct RootState {
    pub app: AppState,
    pub graphics: GraphicsState,
    pub game: GameState,
    pub is_inited: bool,
}

impl RootState {
    pub fn try_new(start_scene_name: &str, window: Arc<Window>) -> Result<Self> {
        let app = AppState::new(Arc::clone(&window));
        let graphics = GraphicsState::try_new(window)?;
        let game = GameState::new();

        Ok(Self {
            app,
            graphics,
            game,
            is_inited: false,
        })
    }
}
