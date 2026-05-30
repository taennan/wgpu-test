use glam::UVec2;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Texture {
    pub path: PathBuf,
    #[serde(default = "Texture::default_divisions")]
    pub divisions: UVec2,
}

impl Texture {
    fn default_divisions() -> UVec2 {
        UVec2::ONE
    }

    pub fn new(path: PathBuf) -> Self {
        Self::new_with_divisions(path, Self::default_divisions())
    }

    pub fn new_with_divisions(path: PathBuf, divisions: UVec2) -> Self {
        let this = Self { path, divisions };
        this.validate();
        this
    }

    fn validate(&self) {
        if self.divisions.x < 1 || self.divisions.y < 1 {
            panic!("Texture divisions must always be greater than 0");
        }
    }
}
