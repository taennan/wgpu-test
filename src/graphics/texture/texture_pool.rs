use super::Texture;
use crate::error::*;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use wgpu::{Device, Queue};

pub struct TexturePool {
    textures: HashMap<PathBuf, Texture>,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl TexturePool {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        Self {
            textures: HashMap::new(),
            queue,
            device,
        }
    }

    pub fn load<P>(&mut self, image_path: &P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let key = self.texture_key(&image_path);
        if self.textures.contains_key(&key) {
            return Ok(());
        }

        let texture = Texture::load(&key, &self.device, &self.queue)?;
        self.textures.insert(key, texture.clone());
        Ok(())
    }

    fn texture_key<P>(&self, image_path: &P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        PathBuf::from(image_path.as_ref())
    }

    pub fn get<P>(&self, image_path: &P) -> Option<&Texture>
    where
        P: AsRef<Path>,
    {
        let key = self.texture_key(image_path);
        self.textures.get(&key)
    }

    pub fn unload<P>(&mut self, image_path: &P)
    where
        P: AsRef<Path>,
    {
        let key = self.texture_key(image_path);
        self.textures.remove(&key);
    }
}
