use crate::graphics::{
    BindGroupSwapper, BufferSwapper,
    camera::{CameraAttributes, CameraBufferData},
};
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BufferUsages, CommandEncoder, Device,
    ShaderStages,
};

pub struct CameraManager {
    pub attributes: CameraAttributes,
    buffer_data: CameraBufferData,
    buffers: BufferSwapper<CameraBufferData>,
    bind_groups: BindGroupSwapper,
}

impl CameraManager {
    pub fn new(device: &Device) -> Self {
        let attributes = CameraAttributes::default();

        let mut buffer_data = CameraBufferData::new();
        buffer_data.set_view_projection(attributes.projection_matrix());

        let buffers = BufferSwapper::<CameraBufferData>::builder()
            .name("Camera")
            .count(2)
            .usages(BufferUsages::UNIFORM)
            .build(CameraBufferData::size() as u64, device);

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        });
        let bind_groups = BindGroupSwapper::builder()
            .name("Camera")
            .resources([buffers.buffer(0).unwrap().as_entire_binding()])
            .resources([buffers.buffer(1).unwrap().as_entire_binding()])
            .build(bind_group_layout, device);

        Self {
            attributes,
            buffer_data,
            buffers,
            bind_groups,
        }
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_groups.layout()
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_groups.current()
    }

    pub fn update_staging_buffer(&mut self) {
        self.buffer_data
            .set_view_projection(self.attributes.projection_matrix());

        self.buffers.write_to_staging(&[self.buffer_data]);
    }

    pub fn update_main_buffer(&mut self, encoder: &mut CommandEncoder) {
        self.buffers.write_to_next(encoder);
        self.buffers.swap();
        self.bind_groups.swap();
    }
}
