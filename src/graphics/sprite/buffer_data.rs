use crate::{
    game::Sprite,
    graphics::{geometry::Vertex, texture::TextureAtlas, types::ColourData},
};
use bytemuck::{Pod, Zeroable};
use glam::{U8Vec2, UVec2, Vec3};
use std::mem;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub type SpriteVertexBufferData = Vertex;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct SpriteInstanceBufferData {
    packed_colour: UVec2,
    position: Vec3,
}

impl SpriteInstanceBufferData {
    pub const SIZE: u64 = mem::size_of::<Self>() as u64;

    pub fn from_sprite(sprite: &Sprite, atlas: &TextureAtlas) -> Self {
        let atlas_item_index = sprite
            .texture_atlas_item_index
            .or(atlas.atlas_item_index(&sprite.texture_path))
            .expect("Failed to get Sprite atlas item index");
        let colour = ColourData::textured(atlas_item_index, U8Vec2::ZERO);

        Self {
            packed_colour: colour.into(),
            position: sprite.position,
        }
    }

    const PACKED_COLOUR_OFFSET: u64 = 0;
    const PACKED_COLOUR_FORMAT: VertexFormat = VertexFormat::Uint32x2;

    const POSITION_OFFSET: u64 = Self::PACKED_COLOUR_OFFSET + Self::PACKED_COLOUR_FORMAT.size();
    const POSITION_FORMAT: VertexFormat = VertexFormat::Float32x3;

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
                offset: Self::POSITION_OFFSET,
                format: Self::POSITION_FORMAT,
            },
        ],
    };
}
