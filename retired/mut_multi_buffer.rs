use super::mut_buffer::MutBuffer;
use crate::utils::rw_lockpick;
use bytemuck::Pod;
use std::{
    marker::PhantomData,
    sync::{Arc, RwLock},
};
use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferUsages, CommandEncoder, Device};

pub struct MutMultiBuffer {
    main: Vec<Buffer>,
    staging: MutBuffer,
    current_index: usize,
    has_staged_lock: Arc<RwLock<bool>>,
    size: BufferAddress,
}

#[derive(Debug)]
pub struct MutMultiBufferBuilder {
    _name: Option<&'static str>,
    _usages: Option<BufferUsages>,
    _count: usize,
}

impl Default for MutMultiBufferBuilder {
    fn default() -> Self {
        Self {
            _name: None,
            _usages: None,
            _count: 2,
        }
    }
}

impl MutMultiBufferBuilder {
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
            panic!("MutMultiBuffer must have at least 2 main buffers");
        }
        self._count = count;
        self
    }

    pub fn build<S, T>(self, size: S, device: &Device) -> MutMultiBuffer
    where
        S: Into<BufferAddress>,
    {
        let size: BufferAddress = size.into();
        let mapped_at_creation = false;

        let staging = MutBuffer::builder()
            .optional_name(self._name)
            .usages(BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC)
            .build(size, device);

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

        MutMultiBuffer {
            staging,
            main: main_buffers,
            current_index: 0,
            has_staged_lock: Arc::new(RwLock::new(false)),
            size,
        }
    }

    fn buffer_name(&self, suffix: &str) -> Option<String> {
        self._name.map(|name| format!("{} {}", name, suffix))
    }
}

impl MutMultiBuffer {
    pub fn builder() -> MutMultiBufferBuilder {
        MutMultiBufferBuilder::default()
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
        let has_staged_data = rw_lockpick::read_or(&self.has_staged_lock, true);
        if !has_staged_data {
            return;
        }

        let offset = 0;
        encoder.copy_buffer_to_buffer(
            self.staging.buffer(),
            offset,
            self.next(),
            offset,
            self.size,
        );

        rw_lockpick::write(&self.has_staged_lock, false);
    }
}

impl MutMultiBuffer {
    pub fn write_to_staging<T>(&mut self, data: &[T])
    where
        T: Pod + Send + Sync,
    {
        let has_staged_data = rw_lockpick::read_or(&self.has_staged_lock, true);
        if has_staged_data || self.staging.is_mapped() {
            log::error!("Buffer is already mapped");
            return;
        }

        let has_staged_lock = self.has_staged_lock.clone();

        self.staging.write_async(
            data,
            move || {
                rw_lockpick::write(&has_staged_lock, true);
            },
            || {},
        );
    }
}
