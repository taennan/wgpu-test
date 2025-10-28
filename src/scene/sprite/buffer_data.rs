use crate::scene::Sprite;
use bytemuck::{Pod, Zeroable};
use std::mem;
use wgpu::{
    Buffer, BufferUsages, Device, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct SpriteVertexBufferData(u32);

impl SpriteVertexBufferData {
    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<SpriteVertexBufferData>() as u64,
        step_mode: VertexStepMode::Vertex,
        attributes: &[VertexAttribute {
            shader_location: 0,
            offset: 0,
            format: VertexFormat::Uint32,
        }],
    };

    pub fn buffer(device: &Device) -> Buffer {
        let buffer_data = [Self(0), Self(1), Self(2), Self(3), Self(4), Self(5)];
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            usage: BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(&buffer_data),
        });
        buffer
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct SpriteInstanceBufferData {
    pub size: [f32; 2],
    pub position: [f32; 3],
    pub texture_division_coords: [u32; 2],
}

impl From<&Sprite> for SpriteInstanceBufferData {
    fn from(sprite: &Sprite) -> Self {
        Self {
            size: sprite.size.into(),
            position: sprite.position.into(),
            texture_division_coords: sprite.texture_division_coords.into(),
        }
    }
}

impl SpriteInstanceBufferData {
    const SIZE_OFFSET: u64 = 0;
    const POS_OFFSET: u64 = Self::SIZE_OFFSET + VertexFormat::Float32x2.size();
    const COORDS_OFFSET: u64 = Self::POS_OFFSET + VertexFormat::Float32x3.size();

    pub const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<SpriteInstanceBufferData>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: &[
            VertexAttribute {
                shader_location: 1,
                offset: Self::SIZE_OFFSET,
                format: VertexFormat::Float32x2,
            },
            VertexAttribute {
                shader_location: 2,
                offset: Self::POS_OFFSET,
                format: VertexFormat::Float32x3,
            },
            VertexAttribute {
                shader_location: 3,
                offset: Self::COORDS_OFFSET,
                format: VertexFormat::Uint32x2,
            },
        ],
    };
}
