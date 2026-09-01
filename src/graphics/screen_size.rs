use crate::graphics::{
    bind_group::{BindGroupBuilder, BindGroupLayoutBuilder},
    buffer::MutBuffer,
};
use glam::UVec2;
use std::mem;
use wgpu::{
    BindGroup, BindGroupLayout, BindingResource, BindingType, BufferBindingType, BufferUsages,
    Device, ShaderStages,
};

pub struct GpuScreenSize {
    _buffer: MutBuffer,
    _bind_group: BindGroup,
    _bind_group_layout: BindGroupLayout,
}

impl GpuScreenSize {
    pub fn new(device: &Device) -> Self {
        let buffer = MutBuffer::builder()
            .name("Screen Size Buffer")
            .usages(BufferUsages::UNIFORM | BufferUsages::MAP_WRITE)
            .build(mem::size_of::<UVec2>() as u64, device);

        let bind_group_layout = BindGroupLayoutBuilder::new()
            .entry(
                ShaderStages::VERTEX,
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )
            .build(device);

        let bind_group = BindGroupBuilder::new()
            .name("Screen Size Bind Group")
            .entry(BindingResource::Buffer(
                buffer.buffer().as_entire_buffer_binding(),
            ))
            .build(&bind_group_layout, device);

        Self {
            _buffer: buffer,
            _bind_group: bind_group,
            _bind_group_layout: bind_group_layout,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self._bind_group
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self._bind_group_layout
    }

    pub fn set_value<T>(&mut self, value: T)
    where
        T: Into<UVec2>,
    {
        let size: UVec2 = value.into();
        self._buffer.write_then(
            Box::from(size),
            move || {
                log::debug!("Setting screen size buffer to {}", size);
            },
            || {},
        );
    }
}
