use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub mesh_path: PathBuf,
    pub texture_path: PathBuf,
    #[serde(default = "Mesh::default_position")]
    pub position: Vec3,
    #[serde(default = "Mesh::default_scale")]
    pub scale: Vec3,
}

impl Mesh {
    fn default_scale() -> Vec3 {
        Vec3::ONE
    }

    fn default_position() -> Vec3 {
        Vec3::ZERO
    }
}
