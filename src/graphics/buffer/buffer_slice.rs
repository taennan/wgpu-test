#[derive(Debug, Eq, PartialEq)]
pub struct BufferSlice {
    pub start: u64,
    pub bytes: Box<[u8]>,
}

impl BufferSlice {
    pub fn new<B>(start: u64, bytes: B) -> Self
    where
        B: Into<Box<[u8]>>,
    {
        Self {
            start,
            bytes: bytes.into(),
        }
    }

    pub fn condensed(mut slices: Vec<BufferSlice>) -> Vec<BufferSlice> {
        slices.sort_by_key(|slice| slice.start);

        let mut condensed = Vec::new();

        let mut active_slice_start = 0;
        let mut active_slice_end = 0;
        let mut active_slice_bytes = Vec::<u8>::new();

        for (index, slice) in slices.into_iter().enumerate() {
            if slice.bytes.is_empty() {
                continue;
            }

            let is_first = index == 0;
            let is_inline = slice.start == active_slice_end;
            if is_first || is_inline {
                if is_first {
                    active_slice_start = slice.start;
                }
                active_slice_end = slice.start + slice.bytes.len() as u64;
                active_slice_bytes.extend(slice.bytes);
                continue;
            }

            let prev_slice = BufferSlice::new(active_slice_start, active_slice_bytes);
            condensed.push(prev_slice);

            active_slice_start = slice.start;
            active_slice_end = slice.start + slice.bytes.len() as u64;
            active_slice_bytes = slice.bytes.to_vec();
        }

        if !active_slice_bytes.is_empty() {
            let last_slice = BufferSlice::new(active_slice_start, active_slice_bytes);
            condensed.push(last_slice);
        }

        condensed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_condenses_inline_slices() {
        for (input, expected) in [
            (
                vec![
                    BufferSlice::new(4, [4, 5, 6]),
                    BufferSlice::new(7, [7, 8, 9]),
                ],
                vec![BufferSlice::new(4, [4, 5, 6, 7, 8, 9])],
            ),
            (
                vec![
                    BufferSlice::new(0, [0, 1, 2]),
                    BufferSlice::new(3, [3, 4, 5]),
                    BufferSlice::new(7, [6, 7, 8]),
                ],
                vec![
                    BufferSlice::new(0, [0, 1, 2, 3, 4, 5]),
                    BufferSlice::new(7, [6, 7, 8]),
                ],
            ),
            (
                vec![
                    BufferSlice::new(0, [0, 1, 2]),
                    BufferSlice::new(4, [4, 5, 6]),
                    BufferSlice::new(9, [9, 10, 11]),
                ],
                vec![
                    BufferSlice::new(0, [0, 1, 2]),
                    BufferSlice::new(4, [4, 5, 6]),
                    BufferSlice::new(9, [9, 10, 11]),
                ],
            ),
        ] {
            let actual = BufferSlice::condensed(input);
            assert_eq!(actual, expected);
        }
    }
}
