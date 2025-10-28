use std::path::PathBuf;

use glam::Vec3;

pub struct Mesh {
    pub data_path: PathBuf,
    pub texture_path: PathBuf,
    pub position: Vec3,
    pub scale: Vec3,
}
