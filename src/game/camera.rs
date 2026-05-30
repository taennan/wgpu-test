use glam::{Mat4, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Camera {
    #[serde(default = "Camera::default_position")]
    pub position: Vec3,
    #[serde(default = "Camera::default_speed")]
    pub speed: f32,
    #[serde(default = "Camera::default_target")]
    pub target: Vec3,
    #[serde(default = "Camera::default_up")]
    pub up: Vec3,
    #[serde(default = "Camera::default_aspect")]
    pub aspect: f32,
    #[serde(default = "Camera::default_field_of_vision_y")]
    pub field_of_vision_y: f32,
    #[serde(default = "Camera::default_cutoff_near")]
    pub cutoff_near: f32,
    #[serde(default = "Camera::default_cutoff_far")]
    pub cutoff_far: f32,
}

///
/// Used to transform 0 - 1 coordinate systems to -1 - 1
///
#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Self::default_position(),
            speed: Self::default_speed(),
            target: Self::default_target(),
            up: Self::default_up(),
            aspect: Self::default_aspect(),
            field_of_vision_y: Self::default_field_of_vision_y(),
            cutoff_near: Self::default_cutoff_near(),
            cutoff_far: Self::default_cutoff_far(),
        }
    }
}

impl Camera {
    fn default_position() -> Vec3 {
        Vec3::Z * 10.0
    }

    fn default_speed() -> f32 {
        0.1
    }

    fn default_target() -> Vec3 {
        Vec3::ZERO
    }

    fn default_up() -> Vec3 {
        Vec3::Y
    }

    fn default_aspect() -> f32 {
        1.0
    }

    fn default_field_of_vision_y() -> f32 {
        45f32.to_radians()
    }

    fn default_cutoff_near() -> f32 {
        0.1
    }

    fn default_cutoff_far() -> f32 {
        100.0
    }

    pub fn new() -> Self {
        Self::default()
    }
}
