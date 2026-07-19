use glam::UVec2;
use std::collections::HashSet;

pub struct SmartBuffer<T, K> {
    pub(self) items: Vec<T>,
    item_size: u64,
    bytes_getter: BytesGetter<T>,
    index_getter: IndexGetter<K>,
    chunk_size: usize,
    dirty_chunks: HashSet<usize>,
}

type BytesGetter<T> = Box<dyn Fn(&T) -> Box<[u8]>>;
type IndexGetter<K> = Box<dyn Fn(&K) -> Option<usize>>;

#[derive(Clone, Copy, Debug)]
enum SetChunkError {
    Dirty,
    Zero,
}

pub struct BufferSlice {
    start: u64,
    data: Box<[u8]>,
}

impl<T, K> SmartBuffer<T, K> {
    pub fn new(
        capacity: usize,
        item_size: u64,
        bytes_getter: BytesGetter<T>,
        index_getter: IndexGetter<K>,
    ) -> Self {
        if capacity == 0 {
            panic!("SmartBuffer capacity cannot be 0");
        }
        if item_size == 0 {
            panic!("SmartBuffer item_size cannot be 0");
        }

        Self {
            items: Vec::<T>::with_capacity(capacity),
            item_size,
            bytes_getter,
            index_getter,
            chunk_size: 1,
            dirty_chunks: HashSet::new(),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty_chunks.len() > 0
    }

    pub fn set_chunk_size(&mut self, chunk_size: usize) -> Result<(), SetChunkError> {
        if self.is_dirty() {
            Err(SetChunkError::Dirty)
        } else if chunk_size == 0 {
            Err(SetChunkError::Zero)
        } else {
            self.chunk_size = chunk_size;
            Ok(())
        }
    }

    pub fn get(&self, key: &K) -> Option<&T> {
        self.item_index(key).map(|i| self.items.get(i))?
    }

    pub fn get_and_dirty(&mut self, key: &K) -> Option<&T> {
        match self.prepare_dirty_get(key) {
            Some(item_index) => self.items.get(item_index),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut T> {
        match self.prepare_dirty_get(key) {
            Some(item_index) => self.items.get_mut(item_index),
            _ => None,
        }
    }

    fn prepare_dirty_get(&mut self, key: &K) -> Option<usize> {
        let (item_index, chunk_index) = match (self.item_index(key), self.chunk_index(key)) {
            (Some(i), Some(c)) => (i, c),
            _ => return None,
        };

        self.dirty_chunks.insert(chunk_index);
        Some(item_index)
    }

    fn item_index(&self, key: &K) -> Option<usize> {
        (self.index_getter)(key).map(|i| if i >= self.items.len() { None } else { Some(i) })?
    }

    fn chunk_index(&self, key: &K) -> Option<usize> {
        let item_index = self.item_index(key);
        if item_index.is_none() {
            return None;
        }

        let item_index = item_index.unwrap();
        let chunk_index = item_index / self.chunk_size;
        Some(chunk_index)
    }

    pub fn dirty_bytes(&mut self) -> Vec<BufferSlice> {
        if !self.is_dirty() {
            return vec![];
        }

        let mut slices = vec![];
        let mut dirty_chunks = self.dirty_chunks.clone().into_iter().collect::<Vec<_>>();
        dirty_chunks.sort();

        for (chunk_index, chunk) in dirty_chunks.iter().enumerate() {
            let last_dirty_chunk = dirty_chunks.get(chunk_index - 1);
            let is_initial = last_dirty_chunk.map_or(true, |c| *c != chunk - 1);

            if is_initial {
                let start = *chunk as u64 * self.chunk_size as u64 * self.item_size;
                let data = Box::new([]);
                slices.push(BufferSlice { start, data });
            }

            let mut data = Vec::with_capacity(
                (self.item_size as usize * self.chunk_size)
                    .try_into()
                    .expect("Failed to convert byte sizes to usize"),
            );
            for item_offset in 0..self.chunk_size {
                let item_index = (*chunk * self.chunk_size) + item_offset;
                let item = self.items.get(item_index);
                if item.is_none() {
                    break;
                }
                let item_bytes = (self.bytes_getter)(item.unwrap());
                data.extend_from_slice(&item_bytes);
            }

            let last_slice_index = slices.len() - 1;
            let slice = slices.get_mut(last_slice_index).unwrap();
            slice.data = data.into_boxed_slice();
        }

        self.clean();
        slices
    }

    pub fn clean(&mut self) {
        self.dirty_chunks.clear();
    }

    fn prepare_fill(&mut self) {
        self.items.clear();
        self.clean();

        let max_chunk_index = self.items.capacity() / self.chunk_size;
        for chunk_index in 0..max_chunk_index {
            self.dirty_chunks.insert(chunk_index);
        }
    }
}

impl<T, K> SmartBuffer<T, K>
where
    T: Clone,
{
    pub fn fill(&mut self, value: &T) {
        self.prepare_fill();
        for _ in 0..self.items.capacity() {
            self.items.push(value.clone());
        }
    }
}

impl<T, K> SmartBuffer<T, K>
where
    T: Default,
{
    pub fn fill_default(&mut self) {
        self.prepare_fill();
        for _ in 0..self.items.capacity() {
            self.items.push(T::default());
        }
    }
}

mod tests {
    use super::*;
    use bytemuck;

    fn mock_bytes_getter<T>(_item: &T) -> Box<[u8]> {
        Box::new([])
    }

    fn mock_index_getter<K>(_key: &K) -> Option<usize> {
        None
    }

    #[test]
    fn it_inits_with_correct_capacity() {
        let buffer = SmartBuffer::<bool, bool>::new(
            10,
            std::mem::size_of::<bool>() as u64,
            Box::new(mock_bytes_getter),
            Box::new(mock_index_getter),
        );
    }

    #[test]
    fn it_dirties_correct_chunks_when_items_are_updated() {
        let mut buffer = SmartBuffer::<u32, usize>::new(
            10,
            std::mem::size_of::<u32>() as u64,
            Box::new(|v| bytemuck::cast_slice(&[*v]).into()),
            Box::new(|k| Some(*k)),
        );
        buffer.set_chunk_size(2).unwrap();
        buffer.fill_default();
        buffer.clean();

        let _ = buffer.get_mut(&0);
        let _ = buffer.get_mut(&2);

        assert_eq!(buffer.dirty_chunks, HashSet::from([0, 1]));
    }

    #[test]
    fn it_cleans_cached_dirty_chunks() {
        let mut buffer = SmartBuffer::<u32, usize>::new(
            10,
            std::mem::size_of::<u32>() as u64,
            Box::new(|v| bytemuck::cast_slice(&[*v]).into()),
            Box::new(|k| Some(*k)),
        );
        buffer.set_chunk_size(2).unwrap();
        buffer.fill_default();

        assert_eq!(buffer.dirty_chunks, HashSet::from([0, 1, 2, 3, 4]));

        buffer.clean();

        assert_eq!(buffer.dirty_chunks.len(), 0);
    }
}
