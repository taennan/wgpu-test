use crate::utils::movement;
use glam::{Mat4, Quat, UVec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Camera {
    pub position: Vec3,
    rotation: Quat,
    up: Vec3,
    pub aspect: f32,
    pub field_of_vision_y: f32,
    pub cutoff_near: f32,
    pub cutoff_far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            up: Vec3::Y,
            aspect: 1.0,
            field_of_vision_y: 45f32.to_radians(),
            cutoff_near: 0.1,
            cutoff_far: 100.0,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_on_screen_resize(&mut self, size: UVec2) {
        let ratio_x = size.x as f32 / size.y as f32;
        self.aspect = ratio_x;
    }

    pub fn get_up(&self) -> Vec3 {
        self.up
    }

    pub fn translate_global(&mut self, offset: Vec3) {
        self.position = movement::translate_global(self.position, offset);
    }

    pub fn translate_local(&mut self, offset: Vec3) {
        self.position = movement::translate_local(self.position, offset, self.rotation)
    }

    // ---- Rotation: set / get / modify ----

    /// Sets rotation directly from Euler angles (radians, XYZ order).
    /// Handy for level-editor style input, but prefer `set_rotation_quat`
    /// or the rotate_by methods for runtime updates to avoid unnecessary
    /// euler->quat conversions.
    pub fn set_rotation(&mut self, euler_angles: Vec3) {
        self.rotation = Quat::from_euler(
            glam::EulerRot::XYZ,
            euler_angles.x,
            euler_angles.y,
            euler_angles.z,
        );
    }

    /// Sets rotation directly from a quaternion.
    pub fn set_rotation_quat(&mut self, rotation: Quat) {
        self.rotation = rotation.normalize();
    }

    /// Gets the current rotation as Euler angles (radians, XYZ order).
    pub fn get_rotation(&self) -> Vec3 {
        let (x, y, z) = self.rotation.to_euler(glam::EulerRot::XYZ);
        Vec3::new(x, y, z)
    }

    /// Gets the current rotation as a raw quaternion.
    pub fn get_rotation_quat(&self) -> Quat {
        self.rotation
    }

    /// Rotates in local space (relative to the camera's own current axes).
    /// This is what you want for mouse-look pitch/yaw: "pitch" always means
    /// tilting relative to where the camera is currently facing.
    pub fn rotate_local(&mut self, angle: Vec3) {
        self.rotation = movement::rotate_local(self.rotation, angle)
    }

    /// Rotates in global/world space (relative to fixed world axes).
    pub fn rotate_global(&mut self, angle: Vec3) {
        self.rotation = movement::rotate_global(self.rotation, angle);
    }

    // ---- Position ----

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    // ---- Derived properties ----

    /// The camera's forward-facing direction, derived from rotation.
    /// By convention here, "forward" is -Z (matches typical OpenGL/wgpu view space).
    pub fn get_forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// The point the camera is currently looking at, one unit along its
    /// forward vector. Useful for constructing a view matrix or for
    /// anything (e.g. debug gizmos) that wants a "look-at point" rather
    /// than a direction.
    pub fn get_target(&self) -> Vec3 {
        self.position + self.get_forward()
    }

    /// Orients the camera to look at `target`, using `self.up` as the
    /// reference for "up" (won't be exactly world-up if the camera ends up
    /// looking straight up/down, since forward and up can't be non-parallel
    /// at that point — see note below).
    pub fn set_target(&mut self, target: Vec3) {
        self.rotation = movement::look_at(self.position, target, self.up).unwrap_or(self.rotation);
    }

    /// The camera's current "up" vector, rotated with the camera.
    /// Falls back to `self.up` only when unrotated; once rotated, up
    /// should follow the camera's orientation, not stay world-fixed.
    fn get_rotated_up(&self) -> Vec3 {
        self.rotation * self.up
    }
}
