use crate::{app::RootState, game::Camera, systems::AppSystem};
use glam::Vec3;
use winit::keyboard::KeyCode;

pub struct CameraMover;

impl AppSystem for CameraMover {
    fn run(&self, state: &mut RootState) {
        let camera = &mut state.game.camera;
        let speed = camera.speed;
        let keys_pressed = &state.app.keys_pressed;

        if keys_pressed.contains(&KeyCode::ArrowLeft) {
            self.rotate_camera_around_origin(-speed, 0.0, camera);
        }
        if keys_pressed.contains(&KeyCode::ArrowRight) {
            self.rotate_camera_around_origin(speed, 0.0, camera);
        }
        if keys_pressed.contains(&KeyCode::ArrowUp) {
            self.rotate_camera_around_origin(0.0, -speed, camera);
        }
        if keys_pressed.contains(&KeyCode::ArrowDown) {
            self.rotate_camera_around_origin(0.0, speed, camera);
        }
    }
}

impl CameraMover {
    fn rotate_camera_around_origin(&self, yaw_delta: f32, pitch_delta: f32, camera: &mut Camera) {
        // Convert current position to spherical coordinates
        let radius = camera.position.length();
        let current_yaw = camera.position.z.atan2(camera.position.x);
        let current_pitch = (camera.position.y / radius).asin();

        // Apply rotation deltas
        let new_yaw = current_yaw + yaw_delta;
        let new_pitch = (current_pitch + pitch_delta).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );

        // Convert back to Cartesian coordinates
        camera.position = Vec3::new(
            radius * new_pitch.cos() * new_yaw.cos(),
            radius * new_pitch.sin(),
            radius * new_pitch.cos() * new_yaw.sin(),
        );
    }
}
