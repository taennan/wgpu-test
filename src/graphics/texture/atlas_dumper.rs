use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crate::graphics::buffer::MutBuffer;
use glam::UVec2;
use image::{Rgba, RgbaImage};
use wgpu::{
    BufferAddress, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, CommandEncoder, Device, Extent3d,
    Origin3d, PollType, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture,
    TextureAspect,
};

pub struct TextureAtlasDumper {
    bytes: Arc<Mutex<[u8]>>,
    buffer: Option<MutBuffer>,
    texture_size: UVec2,
}

impl TextureAtlasDumper {
    pub fn new() -> Self {
        Self {
            bytes: Arc::new(Mutex::new([])),
            buffer: None,
            texture_size: UVec2::ZERO,
        }
    }

    pub fn dump_to_png<P>(
        &mut self,
        filepath: P,
        texture: &Texture,
        device: &Device,
        encoder: &mut CommandEncoder,
    ) -> Result<(), ()>
    where
        P: AsRef<Path>,
    {
        self.dump(texture, device, encoder);

        device
            .poll(PollType::wait_indefinitely())
            .expect("Failed to poll device");

        let bytes = self.bytes.lock().unwrap();
        let mut image = RgbaImage::new(texture.width(), texture.height());

        for y in 0..texture.height() {
            for x in 0..texture.width() {
                let index = (x * y) as usize;
                let r = bytes[index];
                let g = bytes[index + 1];
                let b = bytes[index + 2];
                let a = bytes[index + 3];

                let pixel = image.get_pixel_mut(x, y);
                *pixel = Rgba([r, g, b, a]);
            }
        }

        image
            .save(filepath)
            .expect("Failed to dump TextureAtlas to file");

        Ok(())
    }

    fn dump(&mut self, texture: &Texture, device: &Device, encoder: &mut CommandEncoder) {
        let texture_size = UVec2::ZERO;
        let buffer = Self::init_staging_buffer((texture_size.x * texture_size.y) as u64, device);

        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: buffer.buffer(),
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(COPY_BYTES_PER_ROW_ALIGNMENT),
                    rows_per_image: Some(texture_size.y),
                },
            },
            Extent3d {
                width: 0,
                height: 0,
                depth_or_array_layers: 1,
            },
        );

        buffer.read_then(
            Arc::clone(&self.bytes),
            || {},
            || {
                log::error!("Failed to read TextureAtlas bytes");
            },
        );
    }

    fn init_staging_buffer(size: BufferAddress, device: &Device) -> MutBuffer {
        MutBuffer::builder()
            .name("TextureAtlasDumper Buffer")
            .usages(BufferUsages::COPY_DST | BufferUsages::MAP_READ)
            .build(size, device)
    }
}
