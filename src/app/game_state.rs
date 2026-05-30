use std::path::PathBuf;

use crate::game::{Camera, Mesh, Sprite};

#[derive(Debug, Default)]
pub struct GameState {
    pub scene_path: Option<PathBuf>,
    pub camera: Camera,
    pub sprites: Vec<Sprite>,
    pub meshes: Vec<Mesh>,
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }
}
