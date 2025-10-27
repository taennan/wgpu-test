use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Device,
    ShaderStages,
};

#[derive(Debug, Default)]
pub struct BindGroupLayoutBuilder {
    label: Option<&'static str>,
    entries: Vec<BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, value: &'static str) -> Self {
        self.label = Some(value);
        self
    }

    pub fn entry(mut self, visibility: ShaderStages, ty: BindingType) -> Self {
        self.entries.push(BindGroupLayoutEntry {
            ty,
            visibility,
            binding: self.entries.len() as u32,
            count: None,
        });
        self
    }

    pub fn build(self, device: &Device) -> BindGroupLayout {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: self.label,
            entries: &self.entries,
        });
        layout
    }
}
