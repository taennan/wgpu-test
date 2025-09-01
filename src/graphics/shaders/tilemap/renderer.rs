use super::attributes::TilemapAttributes;
use crate::graphics::texture::Texture;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Device, SamplerBindingType, ShaderStages,
    TextureSampleType, TextureViewDimension,
};

pub struct TilemapRenderer<'a> {
    attributes: &'a TilemapAttributes,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

impl<'a> TilemapRenderer<'a> {
    pub fn new(attributes: &'a TilemapAttributes, texture: &Texture, device: &Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Tilemap Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Tilemap Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.diffuse_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.diffuse_sampler),
                },
            ],
        });

        Self {
            attributes,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }
}
