use crate::graphics::texture::Texture;
use wgpu::{BindGroup as WgpuBindGroup, BindGroupLayout, Device};

pub struct BindGroup {
    layout: BindGroupLayout,
    group: WgpuBindGroup,
}

impl BindGroup {
    pub fn new(texture: &Texture, device: &Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SimpleShape Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    count: None,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                },
            ],
        });
        let group = Self::create_wgpu_bind_group(&layout, texture, device);

        Self { layout, group }
    }

    fn create_wgpu_bind_group(
        layout: &BindGroupLayout,
        texture: &Texture,
        device: &Device,
    ) -> WgpuBindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SimpleShape Bind Group"),
            layout: layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.diffuse_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.diffuse_sampler),
                },
            ],
        })
    }

    pub fn group(&self) -> &WgpuBindGroup {
        &self.group
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    pub fn set_texture(&mut self, texture: &Texture, device: &Device) {
        self.group = Self::create_wgpu_bind_group(&self.layout, texture, device);
    }
}
