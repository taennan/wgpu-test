use crate::{
    app::RootState,
    game::{Camera, Mesh},
    systems::AppSystem,
    utils::paths,
};
use glam::Vec3;
use std::path::PathBuf;

pub struct SceneLoader;

impl AppSystem for SceneLoader {
    fn run(&self, state: &mut RootState) {
        if state.game.scene_path.is_some() {
            return;
        }

        state.game.scene_path = Some(PathBuf::from("test"));

        let mut camera = Camera::new();
        camera.position.y = 10.0;
        camera.target = Vec3::ZERO;
        state.game.camera = camera;

        let mesh_texture_key = paths::texture("albatross-light.jpg");
        let mesh = Mesh::new(paths::geometry("basic-cube.gltf"), mesh_texture_key);
        state.game.meshes = vec![mesh];

        let mut encoder =
            state
                .graphics
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("SceneLoader Encoder"),
                });

        state.graphics.texture_atlas.insert(
            &[],
            &mut state.graphics.texture_buffers,
            &state.graphics.device,
            &mut encoder,
        );

        state.graphics.command_buffers.push(encoder.finish());
    }
}
