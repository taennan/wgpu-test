use crate::vertex::Vertex;
use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix};

pub type ViewProjection = [[f32; 4]; 4];

#[derive(Clone, Debug)]
pub struct TilemapBufferData {
    vertices: Vec<Vertex>,
}

impl TilemapBufferData {
    pub fn new() -> Self {
        Self { vertices: vec![] }
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
