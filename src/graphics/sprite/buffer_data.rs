use crate::{game::Sprite, graphics::texture::TextureAtlas};
use bytemuck::{Pod, Zeroable};
use glam::{UVec2, Vec2, Vec3};
use std::mem;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct SpriteInstanceBufferData {
    scale: Vec2,
    position: Vec3,
    texture_offset: Vec2,
    texture_size: Vec2,
    texture_divisions: UVec2,
    texture_division_coords: UVec2,
}

impl SpriteInstanceBufferData {
    pub const SIZE: u64 = mem::size_of::<Self>() as u64;

    pub fn from_sprite(sprite: &Sprite, atlas: &TextureAtlas) -> Self {
        let texture_uv_data = atlas
            .get_texture_uv_data(&sprite.texture_path)
            .expect("Texture not found");

        Self {
            scale: sprite.scale,
            position: sprite.position,
            texture_offset: texture_uv_data.offset,
            texture_size: texture_uv_data.size,
            texture_division_coords: sprite.texture_division_coords,
            texture_divisions: texture_uv_data.divisions,
        }
    }

    const SCALE_OFFSET: u64 = 0;
    const SCALE_FORMAT: VertexFormat = VertexFormat::Float32x2;

    const POS_OFFSET: u64 = Self::SCALE_OFFSET + Self::SCALE_FORMAT.size();
    const POS_FORMAT: VertexFormat = VertexFormat::Float32x3;

    const TEX_OFFSET_OFFSET: u64 = Self::POS_OFFSET + Self::POS_FORMAT.size();
    const TEX_OFFSET_FORMAT: VertexFormat = VertexFormat::Float32x2;

    const TEX_SIZE_OFFSET: u64 = Self::TEX_OFFSET_OFFSET + Self::TEX_OFFSET_FORMAT.size();
    const TEX_SIZE_FORMAT: VertexFormat = VertexFormat::Float32x2;

    const TEX_DIVS_OFFSET: u64 = Self::TEX_SIZE_OFFSET + Self::TEX_SIZE_FORMAT.size();
    const TEX_DIVS_FORMAT: VertexFormat = VertexFormat::Uint32x2;

    const TEX_DIV_COORDS_OFFSET: u64 = Self::TEX_DIVS_OFFSET + Self::TEX_DIVS_FORMAT.size();
    const TEX_DIV_COORDS_FORMAT: VertexFormat = VertexFormat::Uint32x2;

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<SpriteInstanceBufferData>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: &[
            VertexAttribute {
                shader_location: 2,
                offset: Self::SCALE_OFFSET,
                format: Self::SCALE_FORMAT,
            },
            VertexAttribute {
                shader_location: 3,
                offset: Self::POS_OFFSET,
                format: Self::POS_FORMAT,
            },
            VertexAttribute {
                shader_location: 4,
                format: Self::TEX_OFFSET_FORMAT,
                offset: Self::TEX_OFFSET_OFFSET,
            },
            VertexAttribute {
                shader_location: 5,
                format: Self::TEX_SIZE_FORMAT,
                offset: Self::TEX_SIZE_OFFSET,
            },
            VertexAttribute {
                shader_location: 6,
                format: Self::TEX_DIVS_FORMAT,
                offset: Self::TEX_DIVS_OFFSET,
            },
            VertexAttribute {
                shader_location: 7,
                offset: Self::TEX_DIV_COORDS_OFFSET,
                format: Self::TEX_DIV_COORDS_FORMAT,
            },
        ],
    };
}
