use wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

pub fn align_to_bytes_per_row(image_width: usize) -> usize {
    align_usize(image_width, COPY_BYTES_PER_ROW_ALIGNMENT as usize)
}

fn align_usize(image_width: usize, alignment: usize) -> usize {
    if image_width % alignment == 0 {
        image_width
    } else {
        image_width + (alignment - image_width % alignment)
    }
}
