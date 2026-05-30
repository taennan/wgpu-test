use super::utils;
use glam::{UVec2, UVec3};
use image::{DynamicImage, GenericImageView};
use std::{collections::HashMap, path::PathBuf, u8};
use wgpu::{
    Buffer, BufferUsages, CommandEncoder, Device, Extent3d, Origin3d, TexelCopyBufferInfo,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
    util::{BufferInitDescriptor, DeviceExt},
};

#[derive(Debug, Default)]
pub struct TextureBufferPool {
    buffers: HashMap<PathBuf, TextureBufferData>,
}

#[derive(Debug)]
struct TextureBufferData {
    buffer: Buffer,
    image_size: UVec2,
    padded_bytes_size: UVec2,
}

impl TextureBufferPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: PathBuf, image: &DynamicImage, device: &Device) {
        if self.has(&key) {
            return;
        }

        let bytes_length = 4;
        let rgba = image.to_rgba8();
        let img_x = image.dimensions().0 as usize;
        let img_y = image.dimensions().1 as usize;
        let pad_x = utils::align_to_bytes_per_row(img_x);

        let padded_rgba_length = pad_x * img_y * bytes_length;
        let mut padded_rgba = vec![u8::MAX; padded_rgba_length];

        log::debug!("RGBA Size {}", rgba.len());
        log::debug!("image Size {} {}", img_x, img_y);
        log::debug!("Padded Size {} {}", pad_x, img_y);
        log::debug!("Padded Bytes {}", padded_rgba.len());

        for (row_index, src_row) in rgba.chunks(img_x * bytes_length).enumerate() {
            let dst_row_start = row_index * pad_x * bytes_length;
            let dst_row_width = src_row.len();
            let dst_row_end = dst_row_start + dst_row_width;

            /*
            log::debug!(
                "{} Will copy to slice {}..{} {}",
                row_index,
                dst_row_start,
                dst_row_end,
                dst_row_width
            );
             */

            let dst_row = &mut padded_rgba[dst_row_start..dst_row_end];
            dst_row.copy_from_slice(src_row);
        }

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("{:?} Texture Buffer", key)),
            usage: BufferUsages::COPY_SRC,
            contents: &padded_rgba,
        });

        let data = TextureBufferData {
            buffer,
            image_size: UVec2::new(img_x as u32, img_y as u32),
            padded_bytes_size: UVec2::new((pad_x * bytes_length) as u32, img_y as u32),
        };
        self.buffers.insert(key, data);
    }

    pub fn remove(&mut self, key: &PathBuf) {
        self.buffers.remove(key);
    }

    pub fn has(&self, key: &PathBuf) -> bool {
        self.buffers.contains_key(key)
    }

    pub fn write_to_texture(
        &self,
        key: &PathBuf,
        diffuse_texture: &Texture,
        write_origin: UVec3,
        //write_size: UVec2,
        //queue: &mut Queue,
        encoder: &mut CommandEncoder,
    ) -> Result<(), ()> {
        let buffer_data = self.buffers.get(key).ok_or(())?;

        /*
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &[],
            //&rgba_bytes,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(buffer_data.padded_size.x),
                rows_per_image: Some(buffer_data.padded_size.y),
            },
            copy_size,
        );
         */

        log::debug!(
            "Copying buffer to texture \nPadding Size {}\nBuffer Size {}\nPadded Size Mult {}",
            buffer_data.padded_bytes_size,
            buffer_data.buffer.size(),
            buffer_data.padded_bytes_size.x * buffer_data.padded_bytes_size.y,
        );

        encoder.copy_buffer_to_texture(
            TexelCopyBufferInfo {
                buffer: &buffer_data.buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(buffer_data.padded_bytes_size.x),
                    rows_per_image: Some(buffer_data.padded_bytes_size.y),
                },
            },
            TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: Origin3d {
                    x: write_origin.x,
                    y: write_origin.y,
                    z: write_origin.z,
                },
                aspect: TextureAspect::All,
            },
            Extent3d {
                width: buffer_data.image_size.x,
                height: buffer_data.image_size.y,
                depth_or_array_layers: 1,
            },
        );

        Ok(())
    }
}
