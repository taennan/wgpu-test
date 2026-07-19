use crate::utils::smart_buffer::{BufferSlice, SmartBuffer};
use bytemuck::{Pod, Zeroable};
use glam::{IVec2, UVec2};
use std::{
    iter::Iterator,
    mem,
    ops::{Add, Mul},
};

pub struct Map2D<T> {
    buffer: SmartBuffer<T, BufferKey>,
    width: usize,
    height: usize,
    pub mode: Map2DMode,
    pub offset: IVec2,
}

#[derive(Clone, Copy, Debug)]
pub enum Map2DMode {
    Add,
    Mul,
    Set,
}

type MapKey = u32;
type BufferKey = (MapKey, MapKey);

impl<T> Map2D<T>
where
    T: Clone + Default + Pod + Zeroable,
{
    pub fn new<S>(width: S, height: S) -> Self
    where
        S: Into<usize>,
    {
        let width: usize = width.into();
        let height: usize = height.into();
        if width == 0 || height == 0 {
            panic!("Map2D width and height must not be zero");
        }

        let mut buffer = SmartBuffer::<T, BufferKey>::new(
            width * height,
            mem::size_of::<T>()
                .try_into()
                .expect("Failed to convert usize to u64"),
            Box::new(|v| bytemuck::cast_slice(&[*v]).into()),
            Box::new(move |(x, y)| {
                let x = *x as usize;
                let y = *y as usize;
                if x < width - 1 || y < height - 1 {
                    None
                } else {
                    Some(x * y)
                }
            }),
        );
        buffer.fill_default();
        buffer.clean();

        Self {
            buffer,
            width,
            height,
            mode: Map2DMode::Set,
            offset: IVec2::ZERO,
        }
    }
}

impl<T> Map2D<T> {
    pub fn size(&self) -> UVec2 {
        UVec2::new(self.width as u32, self.height as u32)
    }

    pub fn get(&self, x: MapKey, y: MapKey) -> Option<&T> {
        match self.is_out_of_bounds(x, y) {
            true => None,
            false => self.buffer.get(&(x, y)),
        }
    }

    pub fn get_mut(&mut self, x: MapKey, y: MapKey) -> Option<&mut T> {
        match self.is_out_of_bounds(x, y) {
            true => None,
            false => self.buffer.get_mut(&(x, y)),
        }
    }

    pub fn set(&mut self, x: MapKey, y: MapKey, value: T) {
        if let Some(item) = self.get_mut(x, y) {
            *item = value;
        }
    }

    pub fn is_out_of_bounds(&self, x: MapKey, y: MapKey) -> bool {
        self.size().x < x || self.size().y < y
    }

    pub fn dirty_bytes(&mut self) -> Vec<BufferSlice> {
        self.buffer.dirty_bytes()
    }

    pub fn iter(&self) -> Map2DIter<'_, T> {
        Map2DIter::new(self)
    }

    pub fn iter_mut(&mut self) -> Map2DIterMut<'_, T> {
        Map2DIterMut::new(self)
    }
}

impl<T> Map2D<T>
where
    T: Add<Output = T> + Mul<Output = T> + Clone,
{
    fn apply(&mut self, rhs: &Self) {
        for item in rhs {
            let (item_x, item_y): (i32, i32) = match (item.x.try_into(), item.y.try_into()) {
                (Ok(x), Ok(y)) => (x, y),
                _ => {
                    continue;
                }
            };
            let ix = item_x + rhs.offset.x - self.offset.x;
            let iy = item_y + rhs.offset.y - self.offset.y;

            let (ux, uy): (u32, u32) = match (ix.try_into(), iy.try_into()) {
                (Ok(x), Ok(y)) => (x, y),
                _ => {
                    continue;
                }
            };

            let lhs_item = match self.get_mut(ux, uy) {
                Some(i) => i,
                _ => {
                    continue;
                }
            };

            match rhs.mode {
                Map2DMode::Add => *lhs_item = lhs_item.clone() + item.value.clone(),
                Map2DMode::Mul => *lhs_item = lhs_item.clone() * item.value.clone(),
                Map2DMode::Set => *lhs_item = item.value.clone(),
            };
        }
    }
}

impl<'a, T> IntoIterator for &'a Map2D<T> {
    type Item = Map2DIterItem<'a, T>;
    type IntoIter = Map2DIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

macro_rules! make_map2d_iter {
    ($name:ident, $item_name:ident, $map_method:ident, $($mutability:tt)?) => {
        pub struct $name<'a, T> {
            x: u32,
            y: u32,
            map: *mut Map2D<T>,
            _marker: std::marker::PhantomData<&'a $($mutability)? Map2D<T>>,
        }

        pub struct $item_name<'a, T> {
            pub x: u32,
            pub y: u32,
            pub value: &'a $($mutability)? T,
        }

        impl<'a, T> $name<'a, T> {
            pub fn new(map: &'a $($mutability)? Map2D<T>) -> Self {
                Self {
                    map: map as *const Map2D<T> as *mut Map2D<T>,
                    x: 0,
                    y: 0,
                    _marker: std::marker::PhantomData,
                }
            }
        }

        impl<'a, T> Iterator for $name<'a, T> {
            type Item = $item_name<'a, T>;

            fn next(&mut self) -> Option<Self::Item> {
                let map: &'a $($mutability)? Map2D<T> = unsafe { &$($mutability)? *self.map };

                let size = map.size();
                if self.y >= size.y {
                    return None;
                }

                let (x, y) = (self.x, self.y);
                self.x += 1;
                if self.x >= size.x {
                    self.x = 0;
                    self.y += 1;
                }

                let value = map
                    .$map_method(x, y)
                    .unwrap_or_else(|| panic!("No item at position x={x} y={y}"));

                Some($item_name { x, y, value })
            }
        }
    };
}

make_map2d_iter!(Map2DIter, Map2DIterItem, get,);
make_map2d_iter!(Map2DIterMut, Map2DIterMutItem, get_mut, mut);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_gets_and_sets_items_by_index_correctly() {
        let size = 5usize;
        let mut map = Map2D::new(size, size);

        for (index, position) in [(0, 0), (1, 4), (4, 4), (2, 3), (3, 2)].iter().enumerate() {
            let (x, y) = *position;

            let value = (index as u32) + 10;
            map.set(x, y, value);

            let actual_value = map.get(x, y);
            assert!(!map.is_out_of_bounds(x, y));
            assert!(actual_value.is_some());
            assert_eq!(*actual_value.unwrap(), value);
        }
    }

    #[test]
    fn it_inits_with_correct_size() {
        for (x, y) in [(1usize, 4), (4, 4), (2, 3), (3, 2)].into_iter() {
            let map = Map2D::<u8>::new(x, y);
            let expected = UVec2::new(x as u32, y as u32);
            assert_eq!(map.size(), expected);
        }
    }

    #[test]
    fn it_iterates_items_and_positions() {
        let map = Map2D::<u8>::new(3usize, 3usize);
        let mut iter = map.iter();

        for (x, y) in [
            (0, 0),
            (1, 0),
            (2, 0),
            (0, 1),
            (1, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (2, 2),
        ]
        .into_iter()
        {
            let item = iter.next().unwrap();
            assert_eq!(item.x, x);
            assert_eq!(item.y, y);
            assert_eq!(item.value, map.get(x, y).unwrap());
        }
    }
}
