use crate::{game::mesh::Mesh, graphics::texture::TextureAtlas};
use bytemuck::{Pod, Zeroable};
use glam::{
    //Mat4,
    UVec2,
    Vec3,
};
use std::mem;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct MeshInstanceBufferData {
    position: Vec3,
    atlas_item_index: u32,
    texture_division_coords: UVec2,
}

impl MeshInstanceBufferData {
    pub const SIZE: u64 = mem::size_of::<Self>() as u64;

    pub fn from_mesh(mesh: &Mesh, atlas: &TextureAtlas) -> Self {
        let atlas_item_index = atlas
            .atlas_item_index(&mesh.texture_path)
            .expect("Failed to get Mesh atlas item index");

        Self {
            atlas_item_index,
            position: mesh.position,
            texture_division_coords: UVec2::ZERO,
        }
    }

    const POSITION_OFFSET: u64 = 0;
    const POSITION_FORMAT: VertexFormat = VertexFormat::Float32x3;

    const ATLAS_ITEM_INDEX_OFFSET: u64 = Self::POSITION_OFFSET + Self::POSITION_FORMAT.size();
    const ATLAS_ITEM_INDEX_FORMAT: VertexFormat = VertexFormat::Uint32;

    const TEXTURE_DIVISION_COORDS_OFFSET: u64 =
        Self::ATLAS_ITEM_INDEX_OFFSET + Self::ATLAS_ITEM_INDEX_FORMAT.size();
    const TEXTURE_DIVISION_COORDS_FORMAT: VertexFormat = VertexFormat::Uint32x2;

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<MeshInstanceBufferData>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: &[
            VertexAttribute {
                shader_location: 2,
                offset: Self::POSITION_OFFSET,
                format: Self::POSITION_FORMAT,
            },
            VertexAttribute {
                shader_location: 3,
                offset: Self::ATLAS_ITEM_INDEX_OFFSET,
                format: Self::ATLAS_ITEM_INDEX_FORMAT,
            },
            VertexAttribute {
                shader_location: 4,
                offset: Self::TEXTURE_DIVISION_COORDS_OFFSET,
                format: Self::TEXTURE_DIVISION_COORDS_FORMAT,
            },
        ],
    };
}
