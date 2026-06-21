use glam::UVec2;
use guillotiere::Size;
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

pub fn average_uvec2(sizes: &[UVec2]) -> UVec2 {
    let mut sum = UVec2::ZERO;
    for size in sizes {
        sum += *size;
    }
    sum / sizes.len() as u32
}

pub fn uvec2_to_size(vec: UVec2) -> Size {
    Size::new(vec.x as i32, vec.y as i32)
}
