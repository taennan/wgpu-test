use std::path::PathBuf;
use winit::dpi::PhysicalSize;

pub mod paths {
    use super::*;

    pub fn root() -> PathBuf {
        PathBuf::from(&std::env::current_dir().expect("Couldn't read cwd for some reason"))
            .join("src")
            .to_path_buf()
    }

    pub fn shaders() -> PathBuf {
        root().join("graphics/shaders")
    }

    pub fn assets() -> PathBuf {
        root().join("assets")
    }
}

pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Copy for Vec2<T> where T: Copy {}

impl<T> Clone for Vec2<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.x.clone(), self.y.clone())
    }
}

impl<T> Default for Vec2<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default(), T::default())
    }
}

impl<T> From<(T, T)> for Vec2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y)
    }
}

impl<T> From<PhysicalSize<T>> for Vec2<T> {
    fn from(size: PhysicalSize<T>) -> Self {
        Self::new(size.width, size.height)
    }
}
