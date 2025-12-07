use super::{RendererRenderInput, RendererUpdateInput};
use crate::{
    graphics::{
        geometry::GeometryPool,
        pipeline::{PipelinePool, RenderPassFactory},
        texture::TexturePool,
        texture_v2::TextureAtlas,
    },
    scene::{
        Scene, TilemapRenderer,
        camera::CameraRenderer,
        mesh::MeshRenderer,
        sprite::{Sprite, SpriteRenderer, SpriteRendererUpdateInput},
    },
};
use std::{collections::HashMap, path::PathBuf};
use wgpu::{CommandEncoder, Device, TextureView};

pub struct SceneRenderer {
    pub camera: CameraRenderer,
    pub meshes: MeshRenderer,
    pub sprites: HashMap<PathBuf, SpriteRenderer>,
    pub tilemap: Option<TilemapRenderer>,
}

impl SceneRenderer {
    pub fn new(pipelines: &mut PipelinePool, device: &Device) -> Self {
        let camera = CameraRenderer::new(device);
        let meshes = MeshRenderer::new(&camera, pipelines, device);
        Self {
            camera,
            meshes,
            sprites: HashMap::new(),
            tilemap: None,
        }
    }

    pub fn update(&mut self, scene: &Scene, input: &mut RendererUpdateInput) {
        self.camera.update(&scene.camera);
        self.meshes.update(&scene.meshes, input);

        /*

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
        */
    }

    pub fn render(&self, input: &mut RendererRenderInput) {
        {
            // Must wrap in block so that we can mutably borrow encoder later
            let mut render_pass_factory =
                RenderPassFactory::new(&input.texture_view, input.encoder);
            self.meshes.render(
                self.camera.bind_group(),
                render_pass_factory.start(),
                input.pipelines,
            );
        }

        /*
        for (index, renderer) in self.sprites.values().enumerate() {
            let mut render_pass_factory = RenderPassFactory::new(&texture_view, encoder);
            let render_pass = match index {
                0 => render_pass_factory.start(),
                _ => render_pass_factory.secondary(),
            };

            renderer.render(self.camera.bind_group(), render_pass, pipelines);
        }
         */
    }
}
