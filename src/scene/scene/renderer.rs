use crate::{
    graphics::{
        pipeline::{PipelinePool, RenderPassFactory},
        texture::TexturePool,
    },
    scene::{
        Scene, TilemapRenderer,
        camera::CameraRenderer,
        sprite::{Sprite, SpriteRenderer, SpriteRendererUpdateInput},
    },
};
use std::{collections::HashMap, path::PathBuf};
use wgpu::{CommandEncoder, Device, TextureView};

pub struct SceneRenderer {
    pub camera: CameraRenderer,
    pub sprites: HashMap<PathBuf, SpriteRenderer>,
    pub tilemap: Option<TilemapRenderer>,
}

impl SceneRenderer {
    pub fn new(device: &Device) -> Self {
        Self {
            camera: CameraRenderer::new(device),
            sprites: HashMap::new(),
            tilemap: None,
        }
    }

    pub fn update<'a>(
        &mut self,
        scene: &'a Scene,
        textures: &'a mut TexturePool,
        pipelines: &'a mut PipelinePool,
        device: &'a Device,
    ) {
        // Camera
        self.camera.update(&scene.camera);

        // Sprites
        let mut sorted_sprites = HashMap::<PathBuf, Vec<&Sprite>>::new();

        for sprite in scene.sprites.iter() {
            let key = sprite.texture_path.clone();
            match sorted_sprites.get_mut(&key) {
                Some(sprites) => {
                    sprites.push(sprite);
                }
                _ => {
                    sorted_sprites.insert(key.clone(), vec![sprite]);
                }
            };
        }

        for (key, sprites) in sorted_sprites {
            if !self.sprites.contains_key(&key) {
                self.sprites.insert(
                    key.clone(),
                    SpriteRenderer::new(&sprites, &self.camera, textures, pipelines, device),
                );
            }

            let renderer = self
                .sprites
                .get_mut(&key)
                .expect("Failed to load SpriteRenderer");

            renderer.update(SpriteRendererUpdateInput {
                device,
                sprites: &sprites,
            });
        }
    }

    pub fn render(
        &self,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
        pipelines: &mut PipelinePool,
    ) {
        for (index, renderer) in self.sprites.values().enumerate() {
            let mut render_pass_factory = RenderPassFactory::new(&texture_view, encoder);
            let mut render_pass = match index {
                0 => render_pass_factory.start(),
                _ => render_pass_factory.secondary(),
            };

            renderer.render(self.camera.bind_group(), &mut render_pass, pipelines);
        }
    }
}
