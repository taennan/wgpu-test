use std::sync::RwLock;

pub fn read_or<T>(lock: &RwLock<T>, default_value: T) -> T
where
    T: Clone,
{
    let read_result = lock.read();
    match read_result {
        Ok(data) => data.clone(),
        Err(_) => default_value,
    }
}

pub fn read_or_default<T>(lock: &RwLock<T>) -> T
where
    T: Clone + Default,
{
    let read_result = lock.read();
    match read_result {
        Ok(data) => data.clone(),
        Err(_) => T::default(),
    }
}

pub fn write<T>(lock: &RwLock<T>, value: T) {
    let write_result = lock.write();
    match write_result {
        Ok(mut data) => *data = value,
        Err(_) => log::error!("Failed to write to lock"),
    }
}
