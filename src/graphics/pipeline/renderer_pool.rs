use crate::scene::{SpriteRenderer, TilemapRenderer};
use std::{
    collections::{HashMap, hash_map},
    path::PathBuf,
};

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum RendererKey {
    Sprite(PathBuf),
    Tilemap(PathBuf),
}

pub enum Renderer {
    Sprite(SpriteRenderer),
    Tilemap(TilemapRenderer),
}

pub struct RendererPool {
    renderers: HashMap<RendererKey, Renderer>,
}

impl RendererPool {
    pub fn new() -> Self {
        Self {
            renderers: HashMap::new(),
        }
    }

    pub fn has_sprite(&self, key: &PathBuf) -> bool {
        self.renderers
            .contains_key(&RendererKey::Sprite(key.clone()))
    }

    pub fn get_sprite_mut(&mut self, key: &PathBuf) -> Option<&mut SpriteRenderer> {
        let value = self.renderers.get_mut(&RendererKey::Sprite(key.clone()));
        match value {
            Some(Renderer::Sprite(renderer)) => Some(renderer),
            _ => None,
        }
    }

    pub fn load_sprite(&mut self, key: PathBuf, renderer: SpriteRenderer) {
        self.renderers
            .insert(RendererKey::Sprite(key), Renderer::Sprite(renderer));
    }

    pub fn unload_sprite(&mut self, key: PathBuf) {
        self.renderers.remove(&RendererKey::Sprite(key));
    }

    pub fn values<'a>(&'a self) -> hash_map::Values<'a, RendererKey, Renderer> {
        self.renderers.values()
    }
}
