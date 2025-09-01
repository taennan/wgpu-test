use bytemuck::Pod;
use std::{
    marker::PhantomData,
    mem,
    sync::{Arc, RwLock},
};
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, CommandEncoder, Device, MapMode,
};

pub struct BufferSwapper<T> {
    main: Vec<Buffer>,
    staging: Buffer,
    current_index: usize,
    has_staged_lock: Arc<RwLock<bool>>,
    has_mapped_lock: Arc<RwLock<bool>>,
    _phantom: PhantomData<T>,
}

#[derive(Debug)]
pub struct BufferSwapperBuilder {
    _name: Option<&'static str>,
    _usages: Option<BufferUsages>,
    _count: usize,
}

impl Default for BufferSwapperBuilder {
    fn default() -> Self {
        Self {
            _name: None,
            _usages: None,
            _count: 2,
        }
    }
}

impl BufferSwapperBuilder {
    pub fn name(mut self, name: &'static str) -> Self {
        self._name = Some(name);
        self
    }

    pub fn usages(mut self, usages: BufferUsages) -> Self {
        self._usages = Some(usages);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        if count < 2 {
            panic!("BufferSwapper must have at least 2 main buffers");
        }
        self._count = count;
        self
    }

    pub fn build<S, T>(self, size: S, device: &Device) -> BufferSwapper<T>
    where
        S: Into<BufferAddress>,
    {
        let size: BufferAddress = size.into();
        let mapped_at_creation = false;

        let staging_name = self.buffer_name("Staging Buffer");
        let staging = device.create_buffer(&BufferDescriptor {
            label: staging_name.as_deref(),
            usage: wgpu::BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC,
            mapped_at_creation,
            size,
        });

        let mut main_buffers = Vec::with_capacity(self._count);
        for index in 0..self._count {
            let buffer_name_suffix = format!("Main Buffer {}", index);
            let buffer_name = self.buffer_name(&buffer_name_suffix);

            main_buffers.push(device.create_buffer(&BufferDescriptor {
                label: buffer_name.as_deref(),
                usage: self._usages.unwrap_or(BufferUsages::COPY_DST) | BufferUsages::COPY_DST,
                mapped_at_creation,
                size,
            }));
        }

        BufferSwapper {
            staging,
            main: main_buffers,
            current_index: 0,
            has_staged_lock: Arc::new(RwLock::new(false)),
            has_mapped_lock: Arc::new(RwLock::new(false)),
            _phantom: PhantomData,
        }
    }

    fn buffer_name(&self, suffix: &str) -> Option<String> {
        self._name.map(|name| format!("{} {}", name, suffix))
    }
}

impl<T> BufferSwapper<T> {
    pub fn builder() -> BufferSwapperBuilder {
        BufferSwapperBuilder::default()
    }

    pub fn buffer(&self, index: usize) -> Option<&Buffer> {
        self.main.get(index)
    }

    /*
    pub fn current(&self) -> &Buffer {
        &self.main[self.current_index]
    }
     */

    fn next(&self) -> &Buffer {
        &self.main[self.next_index()]
    }

    pub fn swap(&mut self) {
        self.current_index = self.next_index();
    }

    fn next_index(&self) -> usize {
        (self.current_index + 1) % self.main.len()
    }

    pub fn write_to_next(&self, encoder: &mut CommandEncoder) {
        let has_staged_data = read_bool_lock(&self.has_staged_lock);
        if !has_staged_data {
            return;
        }

        let offset = 0;
        encoder.copy_buffer_to_buffer(
            &self.staging,
            offset,
            self.next(),
            offset,
            self.staging.size(),
        );

        write_bool_lock(&self.has_staged_lock, false);
    }
}

impl<T> BufferSwapper<T>
where
    T: Pod + Send + Sync,
{
    pub fn write_to_staging(&mut self, data: &[T]) {
        let has_staged_data = read_bool_lock(&self.has_staged_lock);
        let has_mapped = read_bool_lock(&self.has_mapped_lock);
        if has_staged_data || has_mapped {
            log::error!("Buffer is already mapped");
            return;
        }

        let data = data.to_vec();
        let staging = self.staging.clone();
        let has_mapped_lock = self.has_mapped_lock.clone();
        let has_staged_lock = self.has_staged_lock.clone();

        write_bool_lock(&has_mapped_lock, true);

        self.staging
            .map_async(MapMode::Write, 0..staging.size(), move |buffer_result| {
                match buffer_result {
                    Ok(_) => {
                        let mut view = staging.get_mapped_range_mut(..);
                        view.copy_from_slice(bytemuck::cast_slice(&data));
                        mem::drop(view);

                        write_bool_lock(&has_staged_lock, true);
                    }
                    Err(_) => {
                        log::error!("Error getting buffer");
                    }
                };

                staging.unmap();
                write_bool_lock(&has_mapped_lock, false);
            });
    }
}

fn read_bool_lock(lock: &Arc<RwLock<bool>>) -> bool {
    let read_result = lock.read();
    match read_result {
        Ok(data) => *data,
        Err(_) => false,
    }
}

fn write_bool_lock(lock: &Arc<RwLock<bool>>, value: bool) {
    let write_result = lock.write();
    match write_result {
        Ok(mut data) => *data = value,
        Err(_) => log::error!("Failed to write to lock"),
    }
}
