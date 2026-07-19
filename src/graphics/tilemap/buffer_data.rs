use crate::{game::Sprite, graphics::texture::TextureAtlas};
use bytemuck::{Pod, Zeroable};
use glam::{UVec2, UVec4, Vec2, Vec3};
use std::mem;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct TilemapVertexBufferData {
    tile_position: UVec2,
    corner: u32,
    colour: UVec4,
}

#[derive(Clone, Copy, Debug)]
pub enum Corner {
    Nw,
    Ne,
    Sw,
    Se,
}

impl From<Corner> for u32 {
    fn from(corner: Corner) -> Self {
        match corner {
            Corner::Nw => 0,
            Corner::Ne => 1,
            Corner::Sw => 2,
            Corner::Se => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TileColour {
    Rgba(UVec4),
    Atlas(UVec2),
}

impl TilemapVertexBufferData {
    pub fn new(tile_position: UVec2, corner: Corner, colour: TileColour) -> Self {
        Self {
            tile_position,
            corner: corner.into(),
            colour: match colour {
                TileColour::Rgba(rgba) => rgba,
                TileColour::Atlas(atlas) => {}
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct TilemapInstanceBufferData {
    map_position: Vec3,
    tile_size: Vec2,
    atlas_item_index: u32,
}

impl TilemapInstanceBufferData {
    pub const SIZE: u64 = mem::size_of::<Self>() as u64;

    pub fn from_sprite(sprite: &Sprite, atlas: &TextureAtlas) -> Self {
        let atlas_item_index = sprite
            .texture_atlas_item_index
            .or(atlas.atlas_item_index(&sprite.texture_path))
            .expect("Failed to get Sprite atlas item index");

        Self {
            atlas_item_index,
            position: sprite.position,
            texture_division_coords: sprite.texture_division_coords,
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
        array_stride: mem::size_of::<SpriteInstanceBufferData>() as u64,
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
