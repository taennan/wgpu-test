use glam::{UVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub name: Option<String>,
    pub mesh_path: PathBuf,
    pub texture_path: PathBuf,
    #[serde(skip)]
    pub texture_atlas_item_index: Option<u32>,
    #[serde(default = "Sprite::default_position")]
    pub position: Vec3,
    #[serde(default = "Sprite::default_scale")]
    pub scale: Vec2,
    #[serde(default = "Sprite::default_texture_division_coords")]
    pub texture_division_coords: UVec2,
}

impl Sprite {
    fn default_position() -> Vec3 {
        Vec3::ZERO
    }

    fn default_scale() -> Vec2 {
        Vec2::ONE
    }

    fn default_texture_division_coords() -> UVec2 {
        UVec2::ZERO
    }

    pub fn new_square(texture_path: PathBuf) -> Self {
        Self::new(
            PathBuf::from("TODO: Add default square mesh with uvs"),
            texture_path,
        )
    }

    pub fn new(mesh_path: PathBuf, texture_path: PathBuf) -> Self {
        Self {
            name: None,
            mesh_path,
            texture_path,
            texture_atlas_item_index: None,
            scale: Self::default_scale(),
            position: Self::default_position(),
            texture_division_coords: Self::default_texture_division_coords(),
        }
    }
}
