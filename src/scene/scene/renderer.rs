use crate::{
    graphics::{
        pipeline::{PipelinePool, Renderer, RendererPool},
        texture::TexturePool,
    },
    scene::{
        Scene,
        camera::CameraRenderer,
        sprite::{Sprite, SpriteRenderer, SpriteRendererRenderInput, SpriteRendererUpdateInput},
    },
};
use std::{collections::HashMap, path::PathBuf};
use wgpu::{CommandEncoder, Device, TextureView};

pub struct SceneRenderer {
    pub camera: CameraRenderer,
    pub renderers: RendererPool,
}

pub struct SceneRendererUpdateInput<'a> {
    pub scene: &'a Scene,
    pub textures: &'a mut TexturePool,
    pub pipelines: &'a mut PipelinePool,
    pub device: &'a Device,
}

pub struct SceneRendererEncodeInput<'a> {
    pub texture_view: &'a TextureView,
    pub encoder: &'a mut CommandEncoder,
    pub pipelines: &'a mut PipelinePool,
}

impl SceneRenderer {
    pub fn new(device: &Device) -> Self {
        Self {
            camera: CameraRenderer::new(device),
            renderers: RendererPool::new(),
        }
    }

    pub fn update<'a>(&mut self, input: SceneRendererUpdateInput<'a>) {
        let mut sorted_sprites = HashMap::<PathBuf, Vec<&Sprite>>::new();

        for sprite in input.scene.sprites.iter() {
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
            if !self.renderers.has_sprite(&key) {
                self.renderers.load_sprite(
                    key.clone(),
                    SpriteRenderer::new(
                        &sprites,
                        &self.camera,
                        input.textures,
                        input.pipelines,
                        input.device,
                    ),
                );
            }

            let renderer = self
                .renderers
                .get_sprite_mut(&key)
                .expect("Failed to load SpriteRenderer");

            renderer.update(SpriteRendererUpdateInput {
                device: input.device,
                sprites: &sprites,
            });
        }

        // Camera
        self.camera.update(&input.scene.camera);
    }

    pub fn encode<'a>(&mut self, input: SceneRendererEncodeInput<'a>) {
        for renderer in self.renderers.values() {
            match renderer {
                Renderer::Sprite(r) => r.render(SpriteRendererRenderInput {
                    camera_bind_group: self.camera.bind_group(),
                    texture_view: input.texture_view,
                    encoder: input.encoder,
                    pipelines: &input.pipelines,
                }),
                Renderer::Tilemap(_) => (),
            };
        }
    }
}
