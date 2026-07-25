//
// Common Shader Utils
// Just copy paste into shaders for now
//

@group(0) @binding(0) var<uniform> screen_size: vec2<u32>;

//
// Atlas
//

@group(1) @binding(0) var atlas_diffuse: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;
@group(1) @binding(2) var<uniform> atlas_padding: u32;
@group(1) @binding(3) var<storage, read> atlas_items: array<AtlasItem>;

struct AtlasItem {
    position: vec2<u32>,
    size: vec2<u32>,
    // X and Y must not be zero!!
    divisions: vec2<u32>,
}

struct AtlasSampleInput {
    packed_colour: vec2<u32>,
    item_index: u32,
    padding: u32,
    division_offset: vec2<f32>,
}

fn atlas_sample(input: AtlasSampleInput) -> vec4<f32>{
    let colour = colour_data_from_vec2(input.packed_colour);

    var atlas_uv_input: AtlasUvInput;
    atlas_uv_input.item_index = input.item_index;
    atlas_uv_input.padding = input.padding;
    atlas_uv_input.division_x = colour.atlas_item_coords.x;
    atlas_uv_input.division_y = colour.atlas_item_coords.y;
    atlas_uv_input.division_offset = input.division_offset;
    let atlas_uv = atlas_uv(atlas_uv_input);
    let atlas_sample = textureSample(atlas_diffuse, atlas_sampler, atlas_uv);

    let blended_sample = vec4(colour.rgba.rgb, atlas_sample.a);

    let sample = select(
        blended_sample,
        select(
            atlas_sample,
            select(
                colour.rgba,
                vec4(0.0),
                !colour.modulate_disabled
            ),
            !colour.atlas_disabled
        ),
        !colour.atlas_disabled && !colour.modulate_disabled
    );
    return sample;
}

struct AtlasUvInput {
    item_index: u32,
    padding: u32,
    division_x: u32,
    division_y: u32,
    division_offset: vec2<f32>,
}

fn atlas_uv(input: AtlasUvInput) -> vec2<f32> {
    let atlas_item = atlas_items[input.item_index];
    let padding_wh = atlas_to_uv_coords(vec2(input.padding, input.padding));

    let total_atlas_item_padding = padding_wh * vec2(f32(atlas_item.position.x + 1), f32(atlas_item.position.x + 1));
    let atlas_item_uv = atlas_to_uv_coords(atlas_item.position) + total_atlas_item_padding;
    let atlas_item_wh = atlas_to_uv_coords(atlas_item.size);

    let division_wh = atlas_item_wh / vec2(f32(atlas_item.divisions.x), f32(atlas_item.divisions.y));
    let division_uv = vec2<f32>(
        (division_wh.x * input.division_offset.x) + (division_wh.x * f32(input.division_x)),
        (division_wh.y * input.division_offset.y) + (division_wh.y * f32(input.division_y))
    );

    return division_uv;
}

fn atlas_to_uv_coords(atlas_coords: vec2<u32>) -> vec2<f32> {
    let atlas_size = textureDimensions(atlas_diffuse);
    return vec2(
        f32(atlas_coords.x) / f32(atlas_size.x),
        f32(atlas_coords.y) / f32(atlas_size.y),
    );
}

struct ColourData {
    atlas_disabled: bool,
    modulate_disabled: bool,
    atlas_item_index: u32,
    atlas_item_coords: vec2<u32>,
    rgba: vec4<f32>,
};

/**
 * ColourData can be stored as a vec2<u32> with the following bit layout.
 * Each section takes 8 bits, except the metadata section, which takes 4
 * and the atlas item index section, which takes 12
 * - metadata (4 bits)
 * - atlas item index (12 bits)
 * - atlas item division x
 * - atlas item division y
 * - red channel
 * - green channel
 * - blue channel
 * - alpha channel
 */
fn colour_data_from_vec2(input: vec2<u32>) -> ColourData {
    let metadata = slice_u32(input.x, 0u, 4u);
    let max_rgba_channel = 255.0;

    var colour: ColourData;
    colour.atlas_disabled = (metadata & 1u) != 0u;
    colour.modulate_disabled = ((metadata >> 1u) & 1u) != 0u;
    colour.atlas_item_index = slice_u32(input.x, 4u, 16u);
    colour.atlas_item_coords.x = slice_u32(input.x, 16u, 24u);
    colour.atlas_item_coords.y = slice_u32(input.x, 24u, 32u);
    colour.rgba.r = f32(slice_u32(input.y, 0u, 8u)) / max_rgba_channel;
    colour.rgba.g = f32(slice_u32(input.y, 8u, 16u)) / max_rgba_channel;
    colour.rgba.b = f32(slice_u32(input.y, 16u, 24u)) / max_rgba_channel;
    colour.rgba.a = f32(slice_u32(input.y, 24u, 32u)) / max_rgba_channel;

    return colour;
}

fn slice_u32(value: u32, start: u32, end: u32) -> u32 {
    let length = 32u;

    let low_bits_filtered = value >> start;
    let high_bits_filtered = low_bits_filtered << (length - end);
    let bits_shifted_to_end = high_bits_filtered >> (length - end);

    return bits_shifted_to_end;
}
