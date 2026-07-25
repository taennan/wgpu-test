use crate::{
    app::RootState,
    game::{Camera, Mesh, Sprite},
    systems::AppSystem,
    utils::paths,
};
use glam::{UVec2, Vec3};
use std::{iter, path::PathBuf};
use wgpu::CommandEncoderDescriptor;
use winit::keyboard::KeyCode;

#[derive(Debug)]
pub struct SceneLoader;

impl AppSystem for SceneLoader {
    fn run(&self, state: &mut RootState) {
        self.run_sprite_and_mesh_test(state);
    }
}

impl SceneLoader {
    fn run_sprite_and_mesh_test(&self, state: &mut RootState) {
        let is_super_pressed = state.app.keys_pressed.contains(&KeyCode::SuperLeft)
            || state.app.keys_pressed.contains(&KeyCode::SuperRight);
        let is_reload_pressed = is_super_pressed && state.app.keys_pressed.contains(&KeyCode::KeyR);
        if is_reload_pressed {
            self.unload_sprite_test(state);
        }
        if state.game.scene_path.is_some() {
            return;
        }

        log::info!("Loading sprite test scene");

        let texture_key_0 = paths::texture("albatross-dark.jpg");
        let texture_key_1 = paths::texture("albatross-light.jpg");

        let square_mesh_key = paths::geometry("square.gltf");
        let cube_mesh_key = paths::geometry("basic-cube.gltf");

        let mut camera = Camera::new();
        camera.position.x = -5.0;
        camera.position.z = -5.0;
        camera.position.y = -30.0;
        camera.set_target(Vec3::ZERO);
        //camera.aspect = 0.5;

        let mesh = Mesh::new(cube_mesh_key.clone(), texture_key_0.clone());

        let mut sprite_0 = Sprite::new(square_mesh_key.clone(), texture_key_0.clone());
        let mut sprite_1 = Sprite::new(square_mesh_key.clone(), texture_key_1.clone());
        sprite_0.position.x = 200.0;
        sprite_1.position.x = -200.0;

        let mut encoder = state
            .graphics
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("SceneLoader Encoder"),
            });

        state.graphics.texture_atlas.insert(
            &[texture_key_0.clone(), texture_key_1.clone()],
            &state.graphics.device,
            &mut state.graphics.queue,
            &mut encoder,
        );
        sprite_0.texture_atlas_item_index = state
            .graphics
            .texture_atlas
            .atlas_item_index(&sprite_0.texture_path);
        sprite_1.texture_atlas_item_index = state
            .graphics
            .texture_atlas
            .atlas_item_index(&sprite_1.texture_path);

        state
            .graphics
            .geometry_pool
            .insert_rect(&square_mesh_key, UVec2::new(200, 150));
        state.graphics.geometry_pool.load(&cube_mesh_key);

        state.game.scene_path = Some(PathBuf::from("Sprite + Mesh Test"));
        state.game.sprites = vec![sprite_0, sprite_1];
        state.game.meshes = vec![mesh];
        state.game.camera = camera;

        state.graphics.queue.submit(iter::once(encoder.finish()));
    }

    fn unload_sprite_test(&self, state: &mut RootState) {
        state.game.scene_path = None;
        state.game.sprites.clear();
        state.game.meshes.clear();
        state.game.camera = Camera::new();
        state.graphics.geometry_pool.clear();
        state.graphics.texture_atlas.clear();
    }
}
