use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
};

#[derive(Debug, Default)]
pub struct BindGroupBuilder<'a> {
    label: Option<String>,
    entries: Vec<BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, value: &str) -> Self {
        self.label = Some(value.to_string());
        self
    }

    pub fn entry(mut self, resource: BindingResource<'a>) -> Self {
        let new_entry = BindGroupEntry {
            binding: self.entries.len() as u32,
            resource,
        };

        self.entries.push(new_entry);
        self
    }

    pub fn build(self, layout: &BindGroupLayout, device: &Device) -> BindGroup {
        let entries = self.entries;
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: self.label.clone().as_deref(),
            layout: &layout,
            entries: &entries,
        });

        bind_group
    }
}
