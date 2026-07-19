use bytemuck::{Pod, Zeroable};
use glam::{UVec2, Vec2, Vec3};
use std::{cmp, mem};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct TilemapVertexBufferData {
    tile_position: UVec2,
    // See tilemap shader for corner values
    corner: u32,
    colour: UVec2,
}

impl TilemapVertexBufferData {
    pub fn new_corners(tile_position: UVec2, colour: &TilemapVertexColourData) -> [Self; 4] {
        [
            Self::new(tile_position, 0, colour),
            Self::new(tile_position, 1, colour),
            Self::new(tile_position, 2, colour),
            Self::new(tile_position, 3, colour),
        ]
    }

    fn new(tile_position: UVec2, corner: u32, colour: &TilemapVertexColourData) -> Self {
        Self {
            tile_position,
            corner,
            colour: colour.into(),
        }
    }

    const TILE_POSITION_OFFSET: u64 = 0;
    const TILE_POSITION_FORMAT: VertexFormat = VertexFormat::Uint32x3;

    const CORNER_OFFSET: u64 = Self::TILE_POSITION_OFFSET + Self::TILE_POSITION_FORMAT.size();
    const CORNER_FORMAT: VertexFormat = VertexFormat::Uint32;

    const COLOUR_OFFSET: u64 = Self::CORNER_OFFSET + Self::CORNER_FORMAT.size();
    const COLOUR_FORMAT: VertexFormat = VertexFormat::Uint32x2;

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: &[
            VertexAttribute {
                shader_location: 0,
                offset: Self::TILE_POSITION_OFFSET,
                format: Self::TILE_POSITION_FORMAT,
            },
            VertexAttribute {
                shader_location: 1,
                offset: Self::CORNER_OFFSET,
                format: Self::CORNER_FORMAT,
            },
            VertexAttribute {
                shader_location: 2,
                offset: Self::COLOUR_OFFSET,
                format: Self::COLOUR_FORMAT,
            },
        ],
    };
}

#[derive(Clone, Debug, Default)]
pub struct TilemapVertexColourData {
    atlas_disabled: bool,
    modulate_disabled: bool,
    atlas_item_division_x: usize,
    atlas_item_division_y: usize,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl TilemapVertexColourData {
    pub fn transparent(self) -> Self {
        Self::default()
    }

    pub fn with_texture(self, atlas_item_division_x: usize, atlas_item_division_y: usize) -> Self {
        Self {
            atlas_disabled: false,
            atlas_item_division_x,
            atlas_item_division_y,
            ..self
        }
    }

    pub fn with_rgba(self, r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            modulate_disabled: false,
            r,
            g,
            b,
            a,
            ..self
        }
    }
}

impl From<&TilemapVertexColourData> for UVec2 {
    fn from(colour: &TilemapVertexColourData) -> Self {
        let atlas_disabled = if colour.atlas_disabled { 0b1 } else { 0b0 };
        let modulate_disabled = if colour.modulate_disabled { 0b10 } else { 0b0 };
        let metadata = 0u16 | atlas_disabled | modulate_disabled;

        let atlas_item_division_x = cmp::min(colour.atlas_item_division_x, u8::MAX as usize) as u8;
        let atlas_item_division_y = cmp::min(colour.atlas_item_division_y, u8::MAX as usize) as u8;

        let atlas_portion = 0u32
            | metadata as u32
            | (atlas_item_division_x as u32)
            | (atlas_item_division_y as u32) << 8;
        let rgba_portion = 0u32
            | (colour.r as u32)
            | (colour.g as u32) << 8
            | (colour.b as u32) << 16
            | (colour.a as u32) << 24;

        UVec2::new(atlas_portion, rgba_portion)
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

    pub fn new(map_position: Vec3, tile_size: Vec2, atlas_item_index: u32) -> Self {
        Self {
            map_position,
            tile_size,
            atlas_item_index,
        }
    }

    const MAP_POSITION_OFFSET: u64 = 0;
    const MAP_POSITION_FORMAT: VertexFormat = VertexFormat::Float32x3;

    const TILE_SIZE_OFFSET: u64 = Self::MAP_POSITION_OFFSET + Self::MAP_POSITION_FORMAT.size();
    const TILE_SIZE_FORMAT: VertexFormat = VertexFormat::Float32x2;

    const ATLAS_ITEM_INDEX_OFFSET: u64 = Self::TILE_SIZE_OFFSET + Self::TILE_SIZE_FORMAT.size();
    const ATLAS_ITEM_INDEX_FORMAT: VertexFormat = VertexFormat::Uint32;

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: &[
            VertexAttribute {
                shader_location: 3,
                offset: Self::MAP_POSITION_OFFSET,
                format: Self::MAP_POSITION_FORMAT,
            },
            VertexAttribute {
                shader_location: 4,
                offset: Self::TILE_SIZE_OFFSET,
                format: Self::TILE_SIZE_FORMAT,
            },
            VertexAttribute {
                shader_location: 5,
                offset: Self::ATLAS_ITEM_INDEX_OFFSET,
                format: Self::ATLAS_ITEM_INDEX_FORMAT,
            },
        ],
    };
}
