use crate::graphics::{
    //geometry::GeometryPool,
    texture::TextureAtlas,
};
use glam::{U8Vec2, UVec2, Vec4};
use std::cmp;
use wgpu::Device;

pub struct RendererUpdateInput<'a> {
    pub texture_atlas: &'a TextureAtlas,
    //pub geometry: &'a GeometryPool,
    //pub pipelines: &'a mut PipelinePool,
    pub device: &'a Device,
    //pub queue: &'a mut Queue,
    //pub encoder: &'a mut CommandEncoder,
}

#[derive(Debug)]
pub struct ColourData {
    pub atlas_disabled: bool,
    pub modulate_disabled: bool,
    pub atlas_item_index: u16,
    pub atlas_item_coords: U8Vec2,
    pub rgba: Vec4,
}

impl ColourData {
    pub fn textured(atlas_item_index: u16, atlas_item_coords: U8Vec2) -> Self {
        Self::new(false, true, atlas_item_index, atlas_item_coords, Vec4::ZERO)
    }

    pub fn solid(rgba: Vec4) -> Self {
        Self::new(true, false, 0, U8Vec2::ZERO, rgba)
    }

    pub fn modulated(atlas_item_index: u16, atlas_item_coords: U8Vec2, rgba: Vec4) -> Self {
        Self::new(false, false, atlas_item_index, atlas_item_coords, rgba)
    }

    fn blank() -> Self {
        Self::new(false, false, 0, U8Vec2::ZERO, Vec4::ZERO)
    }

    fn new(
        atlas_disabled: bool,
        modulate_disabled: bool,
        atlas_item_index: u16,
        atlas_item_coords: U8Vec2,
        rgba: Vec4,
    ) -> Self {
        Self {
            atlas_disabled,
            modulate_disabled,
            atlas_item_index: cmp::min(atlas_item_index, 2u16.pow(14)),
            atlas_item_coords,
            rgba: Vec4::new(
                clamp_to_normalized(rgba.x),
                clamp_to_normalized(rgba.y),
                clamp_to_normalized(rgba.z),
                clamp_to_normalized(rgba.w),
            ),
        }
    }
}

fn clamp_to_normalized(value: f32) -> f32 {
    value.min(1.0).max(0.0)
}

// See shaders for the bit layout of packed colour data
impl From<&ColourData> for UVec2 {
    fn from(value: &ColourData) -> Self {
        let atlas_disabled = if value.atlas_disabled { 1 } else { 0u32 };
        let modulate_disabled = if value.modulate_disabled { 0b10 } else { 0u32 };
        let atlas_item_index = slice_u32(value.atlas_item_index, 0, 12) << 4;
        let atlas_item_coords_x = slice_u32(value.atlas_item_coords.x, 0, 8) << 16;
        let atlas_item_coords_y = slice_u32(value.atlas_item_coords.y, 0, 8) << 24;

        let r = normalized_f32_to_u8(value.rgba.x) as u32;
        let g = (normalized_f32_to_u8(value.rgba.y) as u32) << 8;
        let b = (normalized_f32_to_u8(value.rgba.z) as u32) << 16;
        let a = (normalized_f32_to_u8(value.rgba.w) as u32) << 24;

        let this = UVec2::new(
            atlas_disabled
                | modulate_disabled
                | atlas_item_index
                | atlas_item_coords_x
                | atlas_item_coords_y,
            r | g | b | a,
        );
        this
    }
}

impl From<ColourData> for UVec2 {
    fn from(value: ColourData) -> Self {
        Self::from(&value)
    }
}

fn slice_u32<T>(value: T, start: u32, end: u32) -> u32
where
    T: Into<u32>,
{
    let value = value.into();
    let length = 32;

    let low_bits_filtered = value >> start;
    let high_bits_filtered = low_bits_filtered << (length - end);
    let bits_shifted_to_end = high_bits_filtered >> (length - end);

    return bits_shifted_to_end;
}

fn normalized_f32_to_u8(value: f32) -> u8 {
    (value * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    mod colour_data {
        use super::*;

        #[test]
        fn it_initialises_metadata_correctly() {
            let output = ColourData::textured(0, U8Vec2::ZERO);
            assert!(!output.atlas_disabled);
            assert!(output.modulate_disabled);

            let output = ColourData::solid(Vec4::ZERO);
            assert!(output.atlas_disabled);
            assert!(!output.modulate_disabled);

            let output = ColourData::modulated(0, U8Vec2::ZERO, Vec4::ZERO);
            assert!(!output.atlas_disabled);
            assert!(!output.modulate_disabled);
        }

        #[test]
        fn it_packs_metadata_into_uvec2_correctly() {
            let mut colour = ColourData::blank();
            colour.modulate_disabled = true;
            colour.atlas_disabled = true;

            let uvec = UVec2::from(colour);
            assert_eq!(uvec.x, 0b11);
            assert_eq!(uvec.y, 0b0);
        }

        #[test]
        fn it_packs_into_uvec2_correctly() {
            let colour = ColourData::new(
                true,
                true,
                1,
                U8Vec2::new(5, 10),
                Vec4::new(1.0, 0.0, 1.0, 0.0),
            );

            let uvec = UVec2::from(colour);
            assert_eq!(uvec.x, 0b0000_1010__0000_0101__0000_0000_0001__0011);
            assert_eq!(uvec.y, 0b0000_0000__1111_1111__0000_0000__1111_1111);
        }
    }
}
