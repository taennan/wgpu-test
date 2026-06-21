use bytemuck::{Pod, Zeroable};
use glam::{
    //Mat4,
    UVec2,
};
use std::mem;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct AtlasItemMetaBufferData {
    position: UVec2,
    size: UVec2,
    divisions: UVec2,
}

impl AtlasItemMetaBufferData {
    pub fn new(position: UVec2, size: UVec2, divisions: UVec2) -> Self {
        if divisions.x == 0 || divisions.y == 0 {
            panic!("AtlasItemMetaBufferData divisions must be greater than 0");
        }

        Self {
            position,
            size,
            divisions,
        }
    }

    pub const SIZE: u64 = mem::size_of::<Self>() as u64;

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: Self::SIZE,
        step_mode: VertexStepMode::Instance,
        attributes: &[
            VertexAttribute {
                format: VertexFormat::Uint32x2,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                format: VertexFormat::Uint32x2,
                offset: VertexFormat::Uint32x2.size(),
                shader_location: 1,
            },
            VertexAttribute {
                format: VertexFormat::Uint32x2,
                offset: VertexFormat::Uint32x2.size() * 2,
                shader_location: 2,
            },
        ],
    };
}
