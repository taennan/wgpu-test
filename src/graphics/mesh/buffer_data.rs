use crate::{
    game::mesh::Mesh,
    graphics::{ColourData, geometry::Vertex},
};
use bytemuck::{Pod, Zeroable};
use glam::{UVec2, Vec2, Vec3};
use std::mem;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub type MeshVertexBufferData = Vertex;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct MeshInstanceBufferData {
    packed_colour: UVec2,
    quat_xy: Vec2,
    quat_zw: Vec2,
    position: Vec3,
    _padding: u32,
}

impl From<&Mesh> for MeshInstanceBufferData {
    fn from(mesh: &Mesh) -> Self {
        let colour = ColourData::textured(mesh.texture_atlas_item_index);

        Self {
            packed_colour: colour.into(),
            quat_xy: Vec2::ZERO,
            quat_zw: Vec2::ZERO,
            position: mesh.position,
            _padding: 0,
        }
    }
}

impl MeshInstanceBufferData {
    const PACKED_COLOUR_OFFSET: u64 = 0;
    const PACKED_COLOUR_FORMAT: VertexFormat = VertexFormat::Uint32x2;

    const QUAT_XY_OFFSET: u64 = Self::PACKED_COLOUR_OFFSET + Self::PACKED_COLOUR_FORMAT.size();
    const QUAT_XY_FORMAT: VertexFormat = VertexFormat::Float32x2;

    const QUAT_ZW_OFFSET: u64 = Self::QUAT_XY_OFFSET + Self::QUAT_XY_FORMAT.size();
    const QUAT_ZW_FORMAT: VertexFormat = VertexFormat::Float32x2;

    const POSITION_OFFSET: u64 = Self::QUAT_ZW_OFFSET + Self::QUAT_ZW_FORMAT.size();
    const POSITION_FORMAT: VertexFormat = VertexFormat::Float32x3;

    const PADDING_OFFSET: u64 = Self::POSITION_OFFSET + Self::POSITION_FORMAT.size();
    const PADDING_FORMAT: VertexFormat = VertexFormat::Uint32;

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: &[
            VertexAttribute {
                shader_location: 2,
                offset: Self::PACKED_COLOUR_OFFSET,
                format: Self::PACKED_COLOUR_FORMAT,
            },
            VertexAttribute {
                shader_location: 3,
                offset: Self::QUAT_XY_OFFSET,
                format: Self::QUAT_XY_FORMAT,
            },
            VertexAttribute {
                shader_location: 4,
                offset: Self::QUAT_ZW_OFFSET,
                format: Self::QUAT_ZW_FORMAT,
            },
            VertexAttribute {
                shader_location: 5,
                offset: Self::POSITION_OFFSET,
                format: Self::POSITION_FORMAT,
            },
            VertexAttribute {
                shader_location: 6,
                offset: Self::PADDING_OFFSET,
                format: Self::PADDING_FORMAT,
            },
        ],
    };
}
