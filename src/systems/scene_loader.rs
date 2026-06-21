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

        let texture_key = paths::texture("albatross-dark.jpg");
        let mesh_key = paths::geometry("square.gltf");
        let mut sprite = Sprite::new(mesh_key.clone(), texture_key.clone());

        log::debug!("Will create SceneLoader encoder");
        let mut encoder = state
            .graphics
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("SceneLoader Encoder"),
            });
        log::debug!("Did create SceneLoader encoder");

        state
            .graphics
            .geometry_pool
            .insert_rect(&mesh_key, UVec2::new(100, 50));

        state.graphics.texture_atlas.insert(
            &[texture_key],
            &state.graphics.device,
            &mut state.graphics.queue,
            &mut encoder,
        );
        sprite.texture_atlas_item_index = state
            .graphics
            .texture_atlas
            .atlas_item_index(&sprite.texture_path);

        state.game.scene_path = Some(PathBuf::from("test"));
        state.game.sprites = vec![sprite];
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
