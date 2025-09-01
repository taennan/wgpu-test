use crate::systems::{AppSystem, WindowEventContext};
use cgmath::Vector3;
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct CameraMover {
    speed: f32,
}

impl CameraMover {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }

    fn move_camera(&mut self, vector: Vector3<f32>, ctx: &mut WindowEventContext) {
        ctx.app_state.camera_manager.attributes.position += vector;
        ctx.app_state.camera_manager.update_staging_buffer();
    }
}

impl Default for CameraMover {
    fn default() -> Self {
        Self::new(0.1)
    }
}

impl AppSystem for CameraMover {
    fn handle_window_event(&mut self, ctx: &mut WindowEventContext) {
        match &ctx.event {
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.move_camera(Vector3::unit_x() * -self.speed, ctx);
                }
                PhysicalKey::Code(KeyCode::ArrowRight) => {
                    self.move_camera(Vector3::unit_x() * self.speed, ctx);
                }
                PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.move_camera(Vector3::unit_y() * -self.speed, ctx);
                }
                PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.move_camera(Vector3::unit_y() * self.speed, ctx);
                }
                _ => {}
            },
            _ => {}
        }
    }
}
