#![cfg(test)]

#[test]
fn test_slices() {
    let src_x = 474;
    let src_y = 474;
    let src = vec![0u8; src_x * src_y];

    let dst_x = 512; //src_x + 1;
    let dst_y = src_y; //src_y + 1;
    let mut dst = vec![1u8; dst_x * dst_y];

    assert_eq!(dst.len(), 242688);

    for (i, chunk) in src.chunks(src_x).enumerate() {
        if !(i < src_y as usize) {
            log::error!("{}", i);
            panic!("Somehting went wrong with the row index")
        }
        assert!(i < src_y);
        assert_eq!(chunk, &vec![0u8; src_x]);

        let dst_start = i * dst_x;
        let dst_width = chunk.len();
        let dst_end = dst_start + dst_width;

        log::debug!(
            "{} Will copy to slice {}..{} {}",
            i,
            dst_start,
            dst_end,
            dst_width
        );

        let dst_row = &mut dst[dst_start..dst_end];
        dst_row.copy_from_slice(chunk);
        assert_eq!(dst_row, &vec![0u8; dst_width]);
    }
}
