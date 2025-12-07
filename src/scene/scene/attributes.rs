use crate::scene::{Camera, Sprite, mesh::Mesh, tilemap::attributes::TilemapAttributes};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Scene {
    meta: SceneMeta,
    #[serde(default = "Camera::default")]
    pub camera: Camera,
    pub tilemap: Option<TilemapAttributes>,
    #[serde(default = "Vec::new")]
    pub sprites: Vec<Sprite>,
    #[serde(default = "Vec::new")]
    pub meshes: Vec<Mesh>,
}

#[derive(Serialize, Deserialize)]
pub struct SceneMeta {
    name: String,
    author: String,
}

impl Scene {
    pub fn name(&self) -> &str {
        &self.meta.name
    }
}

impl Default for SceneMeta {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            author: "None".to_string(),
        }
    }
}
