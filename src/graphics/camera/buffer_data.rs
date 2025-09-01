use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix};

pub type ViewProjection = [[f32; 4]; 4];

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraBufferData {
    view_projection: ViewProjection,
}

impl CameraBufferData {
    pub fn new() -> Self {
        Self {
            view_projection: Matrix4::identity().into(),
        }
    }

    pub fn size() -> usize {
        std::mem::size_of::<ViewProjection>()
    }

    pub fn set_view_projection<P>(&mut self, projection: P)
    where
        P: Into<ViewProjection>,
    {
        self.view_projection = projection.into();
    }
}
