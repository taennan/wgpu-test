use crate::{
    error::*,
    utils::{self},
};
use image::{GenericImageView, ImageReader};
use std::path::Path;
use wgpu::{
    Device,
    Queue,
    Sampler,
    //Texture as WgpuTexture,
    TextureDescriptor,
    TextureUsages,
    TextureView,
};

#[derive(Clone, Debug)]
pub struct Texture {
    pub diffuse_view: TextureView,
    pub diffuse_sampler: Sampler,
    //pub diffuse_texture: WgpuTexture,
}

impl Texture {
    pub fn load<P>(asset_path: P, device: &Device, queue: &Queue) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let image_path = utils::paths::assets().join(asset_path);
        let image_name = image_path.display();

        let image = ImageReader::open(&image_path)
            .map_err(|_| Error::AssetLoadingFailed)?
            .decode()
            .map_err(|_| Error::AssetLoadingFailed)?;
        let (width, height) = image.dimensions();

        let diffuse_rgba = image.to_rgba8();

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&TextureDescriptor {
            label: Some(&format!("{}-diffuse-texture", image_name)),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            extent,
        );

        let diffuse_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            diffuse_view,
            diffuse_sampler,
            //diffuse_texture,
        })
    }
}
