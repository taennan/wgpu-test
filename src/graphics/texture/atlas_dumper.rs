use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crate::graphics::buffer::MutBuffer;
use image::RgbaImage;
use wgpu::{
    BufferUsages, CommandEncoderDescriptor, Device, Extent3d, Origin3d, PollType, Queue,
    TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
};

pub struct TextureAtlasDumper;

impl TextureAtlasDumper {
    pub fn new() -> Self {
        Self
    }

    pub fn dump_to_png<P>(&mut self, filepath: P, texture: &Texture, device: &Device, queue: &Queue)
    where
        P: AsRef<Path>,
    {
        let bytes = self.dump(texture, device, queue);
        let image = RgbaImage::from_raw(texture.width(), texture.height(), bytes)
            .expect("Failed to fit bytes into image");

        image
            .save(filepath)
            .expect("Failed to dump TextureAtlas to file");
    }

    fn dump(&mut self, texture: &Texture, device: &Device, queue: &Queue) -> Vec<u8> {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("TextureAtlasDumper Encoder"),
            ..Default::default()
        });

        let buffer = MutBuffer::builder()
            .name("TextureAtlasDumper Buffer")
            .usages(BufferUsages::COPY_DST | BufferUsages::MAP_READ)
            .build(texture.size().width * 4 * texture.size().height, device);

        log::debug!(
            "Will try to copy texture to buffer, texture_size={:?}",
            texture.size()
        );

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
                    bytes_per_row: Some(texture.size().width * 4), //Some(COPY_BYTES_PER_ROW_ALIGNMENT),
                    rows_per_image: Some(texture.size().height),
                },
            },
            Extent3d {
                width: texture.size().width,
                height: texture.size().height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit([encoder.finish()]);

        let bytes = Arc::new(Mutex::new(vec![]));

        log::debug!("Will try read mapped buffer");

        buffer.read_then(
            Arc::clone(&bytes),
            || {},
            || {
                log::error!("Failed to read TextureAtlas bytes");
            },
        );

        log::debug!("Will poll, total_bytes={}", bytes.lock().unwrap().len());
        device
            .poll(PollType::wait_indefinitely())
            .expect("Failed to poll device");

        log::debug!("Will submit, total_bytes={}", bytes.lock().unwrap().len());
        //queue.submit([encoder.finish()]);

        log::debug!("Total bytes copied={}", bytes.lock().unwrap().len());

        Arc::try_unwrap(bytes)
            .expect("Failed to unwrap Arc as it has more than 1 strong reference")
            .into_inner()
            .expect("Failed to get inner value as this Mutex was poisoned")
    }
}
