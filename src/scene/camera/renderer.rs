use std::mem;

use crate::{
    graphics::{
        bind_group::{BindGroupBuilder, BindGroupLayoutBuilder},
        buffer::mut_buffer::MutBuffer,
    },
    scene::Camera,
};
use wgpu::{
    BindGroup, BindGroupLayout, BindingResource, BindingType, BufferBindingType, BufferUsages,
    Device, ShaderStages,
};

pub struct CameraRenderer {
    buffer: MutBuffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

type ViewProjection = [[f32; 4]; 4];

impl CameraRenderer {
    pub fn new(device: &Device) -> Self {
        let buffer = MutBuffer::builder()
            .name("Camera Buffer")
            .usages(BufferUsages::VERTEX | BufferUsages::MAP_WRITE | BufferUsages::UNIFORM)
            .build(mem::size_of::<ViewProjection>() as u64, device);

        let bind_group_layout = BindGroupLayoutBuilder::new()
            .name("Camera Bind Group Layout")
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
            .name("Camera Bind Group")
            .entry(BindingResource::Buffer(
                buffer.buffer().as_entire_buffer_binding(),
            ))
            .build(&bind_group_layout, device);

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn update(&mut self, camera: &Camera) {
        let buffer_data = camera.projection_matrix().to_cols_array_2d();
        self.buffer.write_async(
            &buffer_data,
            || {},
            || log::error!("Failed to write to Camera Buffer"),
        );
    }
}
