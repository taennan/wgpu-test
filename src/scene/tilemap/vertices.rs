use super::attributes::TilemapAttributes;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

#[derive(Clone)]
pub struct TilemapVertices;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TilemapVertex {
    tile_position: [u32; 3],
    tile_texture_index: u32,
    corner: u32,
}

#[derive(Clone, Copy, Debug)]
enum Corner {
    Nw,
    Ne,
    Sw,
    Se,
}

impl Corner {
    pub fn all() -> [Self; 4] {
        [Self::Nw, Self::Ne, Self::Sw, Self::Se]
    }
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

impl TilemapVertices {
    pub fn new() -> Self {
        Self
    }

    pub fn vertex_buffer(&self, attributes: &TilemapAttributes, device: &Device) -> Buffer {
        let mut vertices = Vec::<TilemapVertex>::new();
        for tile in attributes.iter_tiles() {
            for corner in Corner::all() {
                let vertex = TilemapVertex {
                    tile_position: [
                        tile.tile_global_position.x as u32,
                        tile.tile_global_position.y as u32,
                        tile.tile_global_position.z as u32,
                    ],
                    tile_texture_index: tile.tile,
                    corner: corner.into(),
                };
                vertices.push(vertex);
            }
        }

        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Shape Index Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        })
    }
}
