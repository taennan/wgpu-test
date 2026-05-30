use crate::graphics::geometry::Vertex;

#[derive(Clone, Debug, Default)]
pub struct Geometry {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Geometry {
    pub fn with_capacities(vertices: usize, indices: usize) -> Self {
        Geometry {
            vertices: Vec::with_capacity(vertices),
            indices: Vec::with_capacity(indices),
        }
    }
}
