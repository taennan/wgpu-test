use glam::{UVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub name: Option<String>,
    pub texture_path: PathBuf,
    #[serde(default = "Sprite::default_position")]
    pub position: Vec3,
    #[serde(default = "Sprite::default_size")]
    pub size: Vec2,
    #[serde(default = "Sprite::default_texture_divisions")]
    pub texture_divisions: UVec2,
    #[serde(default = "Sprite::default_texture_division_coords")]
    pub texture_division_coords: UVec2,
}

impl Sprite {
    fn default_position() -> Vec3 {
        Vec3::ZERO
    }

    fn default_size() -> Vec2 {
        Vec2::ONE * 5.0
    }

    fn default_texture_divisions() -> UVec2 {
        UVec2::ONE
    }

    fn default_texture_division_coords() -> UVec2 {
        UVec2::ZERO
    }

    pub fn new(texture_path: PathBuf) -> Self {
        Self {
            name: None,
            texture_path,
            size: Self::default_size(),
            position: Self::default_position(),
            texture_divisions: Self::default_texture_divisions(),
            texture_division_coords: Self::default_texture_division_coords(),
        }
    }
}
