use crate::graphics::bind_group::{BindGroupBuilder, BindGroupLayoutBuilder};
use wgpu::{
    BindGroup, BindGroupLayout, BindingResource, BindingType, Device, Sampler, SamplerBindingType,
    ShaderStages, TextureSampleType, TextureView, TextureViewDimension,
};

#[derive(Clone, Debug)]
pub struct AtlasBindings {
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl AtlasBindings {
    pub fn new(texture_view: &TextureView, sampler: &Sampler, device: &Device) -> Self {
        let bind_group_layout = BindGroupLayoutBuilder::new()
            .name("Texture Atlas Bind Group Layout")
            .entry(
                ShaderStages::FRAGMENT,
                BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
            )
            .entry(
                ShaderStages::FRAGMENT,
                BindingType::Sampler(SamplerBindingType::Filtering),
            )
            .build(device);

        let bind_group = BindGroupBuilder::new()
            .name("Mesh Texture Bind Group")
            .entry(BindingResource::TextureView(texture_view))
            .entry(BindingResource::Sampler(sampler))
            .build(&bind_group_layout, device);

        Self {
            bind_group_layout,
            bind_group,
        }
    }
}
