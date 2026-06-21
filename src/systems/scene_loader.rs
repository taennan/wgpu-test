use crate::{
    app::RootState,
    game::{Camera, Mesh, Sprite},
    systems::AppSystem,
    utils::paths,
};
use glam::{UVec2, Vec3};
use std::path::PathBuf;
use wgpu::CommandEncoderDescriptor;

#[derive(Debug)]
pub struct SceneLoader;

impl AppSystem for SceneLoader {
    fn run(&self, state: &mut RootState) {
        self.run_sprite_test(state);
    }
}

impl SceneLoader {
    fn run_sprite_test(&self, state: &mut RootState) {
        if state.game.scene_path.is_some() {
            return;
        }

        let texture_key_0 = paths::texture("albatross-dark.jpg");
        let texture_key_1 = paths::texture("albatross-light.jpg");
        log::debug!("Will create SceneLoader encoder");
        let mut encoder = state
            .graphics
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("SceneLoader Encoder"),
            });
        log::debug!("Did create SceneLoader encoder");

        state.graphics.texture_atlas.insert(
            &[texture_key_0.clone(), texture_key_1.clone()],
            &state.graphics.device,
            &mut state.graphics.queue,
            &mut encoder,
        );

        let mesh_key = paths::geometry("square.gltf");
        state
            .graphics
            .geometry_pool
            .insert_rect(&mesh_key, UVec2::new(200, 150));

        let mut sprite_0 = Sprite::new(mesh_key.clone(), texture_key_0.clone());
        let mut sprite_1 = Sprite::new(mesh_key.clone(), texture_key_1.clone());
        sprite_0.position.x = 200.0;
        sprite_1.position.x = -200.0;

        sprite_0.texture_atlas_item_index = state
            .graphics
            .texture_atlas
            .atlas_item_index(&sprite_0.texture_path);
        sprite_1.texture_atlas_item_index = state
            .graphics
            .texture_atlas
            .atlas_item_index(&sprite_1.texture_path);

        state.game.scene_path = Some(PathBuf::from("test"));
        state.game.sprites = vec![sprite_0, sprite_1];
        state.graphics.command_buffers.push(encoder.finish());
    }

    fn run_mesh_test(&self, state: &mut RootState) {
        if state.game.scene_path.is_some() {
            return;
        }

        state.game.scene_path = Some(PathBuf::from("test"));

        let mut camera = Camera::new();
        camera.position.y = -10.0;
        camera.target = Vec3::ZERO;
        state.game.camera = camera;

        let mesh_texture_key = paths::texture("albatross-light.jpg");
        let mesh = Mesh::new(paths::geometry("basic-cube.gltf"), mesh_texture_key.clone());
        state.game.meshes = vec![mesh];

        log::debug!("Will create SceneLoader encoder");
        let mut encoder = state
            .graphics
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("SceneLoader Encoder"),
            });
        log::debug!("Did create SceneLoader encoder");

        state.graphics.texture_atlas.insert(
            &[mesh_texture_key],
            &state.graphics.device,
            &mut state.graphics.queue,
            &mut encoder,
        );

        state.graphics.command_buffers.push(encoder.finish());
    }
}
