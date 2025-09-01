use cgmath::{Matrix4, Point3, Vector3, Vector4};

pub struct CameraAttributes {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub field_of_vision_y: f32,
    pub cutoff_near: f32,
    pub cutoff_far: f32,
}

///
/// Used to transform 0 - 1 coordinate systems to -1 - 1
///
#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::from_cols(
    Vector4::new(1.0, 0.0, 0.0, 0.0),
    Vector4::new(0.0, 1.0, 0.0, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 0.0),
    Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl Default for CameraAttributes {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::unit_y(),
            aspect: 1.0,
            field_of_vision_y: 45.0,
            cutoff_near: 0.1,
            cutoff_far: 100.0,
        }
    }
}

impl CameraAttributes {
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.position, self.target, self.up);
        let projection = cgmath::perspective(
            cgmath::Deg(self.field_of_vision_y),
            self.aspect,
            self.cutoff_near,
            self.cutoff_far,
        );

        OPENGL_TO_WGPU_MATRIX * projection * view
    }
}
