use crate::{app::AppState, scene::Camera, systems::AppSystem};
use glam::Vec3;
use winit::keyboard::KeyCode;

pub struct CameraMover;

impl AppSystem for CameraMover {
    fn run(&self, state: &mut AppState) {
        let camera = &mut state.scene.camera;
        let speed = camera.speed;

        if state.keys_pressed.contains(&KeyCode::ArrowLeft) {
            self.move_camera(Vec3::X * -speed, camera);
        }
        if state.keys_pressed.contains(&KeyCode::ArrowRight) {
            self.move_camera(Vec3::X * speed, camera);
        }
        if state.keys_pressed.contains(&KeyCode::ArrowUp) {
            self.move_camera(Vec3::Y * -speed, camera);
        }
        if state.keys_pressed.contains(&KeyCode::ArrowDown) {
            self.move_camera(Vec3::Y * speed, camera);
        }
    }
}

impl CameraMover {
    fn move_camera(&self, vector: Vec3, camera: &mut Camera) {
        camera.position += vector;
        //state.camera_manager.update_staging_buffer();
    }
}
