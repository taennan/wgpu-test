use crate::{graphics::texture_v2::TextureAtlas, scene::mesh::Mesh};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2};
use std::mem;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct MeshInstanceBufferData {
    //quaternion: Mat4,
    texture_offset: Vec2,
    texture_size: Vec2,
}

impl MeshInstanceBufferData {
    pub fn from_mesh(mesh: &Mesh, atlas: &TextureAtlas) -> Self {
        let texture_offset = atlas
            .get_texture_uv_offset(&mesh.texture_path)
            .expect("Texture not found");
        let texture_size = atlas
            .get_texture_uv_size(&mesh.texture_path)
            .expect("Texture not found");
        Self {
            //quaternion: Mat4::from_cols(Vec4::ZERO, Vec4::ZERO, Vec4::ZERO, Vec4::ZERO),
            texture_offset,
            texture_size,
        }
    }
}

impl MeshInstanceBufferData {
    pub const SIZE: u64 = mem::size_of::<Self>() as u64;

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: Self::SIZE,
        step_mode: VertexStepMode::Instance,
        attributes: &[
            /*
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 2,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: VertexFormat::Float32x4.size(),
                shader_location: 3,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: VertexFormat::Float32x4.size() * 2,
                shader_location: 4,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: VertexFormat::Float32x4.size() * 3,
                shader_location: 5,
            },
            */
            VertexAttribute {
                format: VertexFormat::Float32x2,
                //offset: VertexFormat::Float32x4.size() * 4,
                offset: 0,
                shader_location: 2,
            },
            VertexAttribute {
                format: VertexFormat::Float32x2,
                //offset: VertexFormat::Float32x4.size() * 4 + VertexFormat::Uint32x2.size(),
                offset: VertexFormat::Float32x2.size(),
                shader_location: 3,
            },
        ],
    };
}
