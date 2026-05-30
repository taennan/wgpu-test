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
    pub fn new<P>(mesh_path: P, texture_path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            mesh_path: mesh_path.into(),
            texture_path: texture_path.into(),
            position: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }

    fn default_scale() -> Vec3 {
        Vec3::ONE
    }

    fn default_position() -> Vec3 {
        Vec3::ZERO
    }
}
