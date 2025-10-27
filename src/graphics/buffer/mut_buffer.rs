use crate::utils::rw_lockpick;
use bytemuck::Pod;
use std::{
    mem,
    sync::{Arc, RwLock},
};
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, Device, MapMode, WasmNotSend,
    util::{BufferInitDescriptor, DeviceExt},
};

#[derive(Clone, Debug)]
pub struct MutBuffer {
    inner: Buffer,
    has_mapped_lock: Arc<RwLock<bool>>,
}

#[derive(Debug)]
pub struct MutBufferBuilder {
    _name: Option<&'static str>,
    _usages: Option<BufferUsages>,
}

impl Default for MutBufferBuilder {
    fn default() -> Self {
        Self {
            _name: None,
            _usages: None,
        }
    }
}

impl MutBufferBuilder {
    pub fn name(mut self, name: &'static str) -> Self {
        self._name = Some(name);
        self
    }

    pub fn optional_name(mut self, name: Option<&'static str>) -> Self {
        if name.is_some() {
            self._name = name;
        }
        self
    }

    pub fn usages(mut self, usages: BufferUsages) -> Self {
        self._usages = Some(usages);
        self
    }

    pub fn build<S>(self, size: S, device: &Device) -> MutBuffer
    where
        S: Into<BufferAddress>,
    {
        let inner = device.create_buffer(&BufferDescriptor {
            label: self._name.as_deref(),
            usage: self._usages.unwrap_or(BufferUsages::MAP_WRITE),
            mapped_at_creation: false,
            size: size.into(),
        });

        MutBuffer {
            inner,
            has_mapped_lock: Arc::new(RwLock::new(false)),
        }
    }

    pub fn build_init<T>(self, contents: &[T], device: &Device) -> MutBuffer
    where
        T: Pod,
    {
        let inner = device.create_buffer_init(&BufferInitDescriptor {
            label: self._name.as_deref(),
            contents: bytemuck::cast_slice(&contents),
            usage: self._usages.unwrap_or(BufferUsages::MAP_WRITE),
        });

        MutBuffer {
            inner,
            has_mapped_lock: Arc::new(RwLock::new(false)),
        }
    }
}

impl MutBuffer {
    pub fn builder() -> MutBufferBuilder {
        MutBufferBuilder::default()
    }

    pub fn buffer(&self) -> &Buffer {
        &self.inner
    }

    pub fn is_mapped(&self) -> bool {
        rw_lockpick::read_or(&self.has_mapped_lock, true)
    }
}

impl MutBuffer {
    pub fn write<T>(&mut self, data: &[T])
    where
        T: Pod + Send + Sync,
    {
        self.write_async(data, || {}, || {});
    }

    pub fn write_async<T, F, E>(&mut self, data: &[T], ok_callback: F, err_callback: E)
    where
        T: Pod + Send + Sync,
        F: FnOnce() + WasmNotSend + 'static,
        E: FnOnce() + WasmNotSend + 'static,
    {
        if self.is_mapped() {
            log::error!("Buffer is already mapped");
            return;
        }

        let data = data.to_vec();
        let inner = self.inner.clone();
        let has_mapped_lock = self.has_mapped_lock.clone();

        rw_lockpick::write(&has_mapped_lock, true);

        self.inner
            .map_async(MapMode::Write, 0..inner.size(), move |buffer_result| {
                match buffer_result {
                    Ok(_) => {
                        let mut view = inner.get_mapped_range_mut(..);
                        view.copy_from_slice(bytemuck::cast_slice(&data));
                        mem::drop(view);

                        ok_callback();
                    }
                    Err(_) => {
                        err_callback();
                    }
                };

                inner.unmap();
                rw_lockpick::write(&has_mapped_lock, false);
            });
    }
}
