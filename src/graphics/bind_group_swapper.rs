use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
};

pub struct BindGroupSwapper {
    layout: BindGroupLayout,
    bind_groups: Vec<BindGroup>,
    current_index: usize,
}

pub struct BindGroupSwapperBuilder<'a, const N: usize> {
    _name: Option<&'static str>,
    _resources: Vec<[BindingResource<'a>; N]>,
}

impl<'a, const N: usize> Default for BindGroupSwapperBuilder<'a, N> {
    fn default() -> Self {
        Self {
            _name: None,
            _resources: Vec::new(),
        }
    }
}

impl<'a, const N: usize> BindGroupSwapperBuilder<'a, N> {
    pub fn name(mut self, name: &'static str) -> Self {
        self._name = Some(name);
        self
    }

    pub fn resources(mut self, resources: [BindingResource<'a>; N]) -> Self {
        self._resources.push(resources);
        self
    }

    pub fn build(self, layout: BindGroupLayout, device: &Device) -> BindGroupSwapper {
        if self._resources.is_empty() {
            panic!("BindGroupSwapperBuilder::build() called with no resources");
        }

        let mut bind_groups = vec![];
        for (index, resources) in self._resources.into_iter().enumerate() {
            let bind_group_name = self
                ._name
                .map(|name| format!("{} Bind Group {}", name, index));
            let entries = resources
                .into_iter()
                .enumerate()
                .map(|(index, resource)| BindGroupEntry {
                    binding: index as u32,
                    resource,
                })
                .collect::<Vec<_>>();

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: bind_group_name.as_deref(),
                layout: &layout,
                entries: &entries,
            });
            bind_groups.push(bind_group);
        }

        BindGroupSwapper {
            layout,
            bind_groups,
            current_index: 0,
        }
    }
}

impl BindGroupSwapper {
    pub fn builder<'a, const N: usize>() -> BindGroupSwapperBuilder<'a, N> {
        BindGroupSwapperBuilder::default()
    }

    pub fn swap(&mut self) {
        self.current_index = self.next_index();
    }

    pub fn current(&self) -> &BindGroup {
        &self.bind_groups[self.current_index]
    }

    fn next_index(&self) -> usize {
        (self.current_index + 1) % self.bind_groups.len()
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}
