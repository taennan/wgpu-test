use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use std::{
    hash::{Hash, Hasher},
    mem,
};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.uv == other.uv
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let pos_values = [self.position.x, self.position.y, self.position.z];
        let pos_bits: &[u8] = bytemuck::cast_slice(&pos_values);
        pos_bits.hash(state);

        let uv_values = [self.uv.x, self.uv.y];
        let uv_bits: &[u8] = bytemuck::cast_slice(&uv_values);
        uv_bits.hash(state);
    }
}

impl Vertex {
    pub const SIZE: u64 = mem::size_of::<Self>() as u64;

    pub const LAYOUT: VertexBufferLayout<'_> = VertexBufferLayout {
        array_stride: Self::SIZE,
        step_mode: VertexStepMode::Vertex,
        attributes: &[
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: VertexFormat::Float32x3.size(),
                shader_location: 1,
            },
        ],
    };
}
