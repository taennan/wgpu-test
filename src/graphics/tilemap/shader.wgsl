//
// Tilemap Shader
//

@group(0) @binding(0) var<uniform> screen_size: vec2<u32>;

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

const NW_CORNER = 0;
const NE_CORNER = 1 ;
const SW_CORNER = 2;
const SE_CORNER = 3;

struct VertexInput {
    // Per vertex
    @location(0) tile_pos: vec3<u32>,
    @location(1) corner: u32,
    @location(2) colour: vec2<u32>,
    // Per instance
    @location(3) map_pos: vec3<f32>,
    @location(4) tile_size: vec2<f32>,
    @location(5) atlas_item_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) colour: vec2<u32>,
    @location(1) corner: u32,
    @location(2) atlas_item_index: u32,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.colour = input.colour;
    out.corner = input.corner;
    out.atlas_item_index = input.atlas_item_index;

    let is_end_corner = input.corner == NE_CORNER || input.corner == SE_CORNER;
    out.position = vec4(
        _vertex_position(input.tile_pos.x, input.tile_size.x, is_end_corner, input.map_pos.x, screen_size.x),
        _vertex_position(input.tile_pos.y, input.tile_size.y, is_end_corner, input.map_pos.y, screen_size.y),
        input.map_pos.z,
        1.0
    );

    return out;
}

fn _vertex_position(tile_pos: u32, tile_size: f32, extend_to_end: bool, map_pos: f32, screen_dimension: u32) -> f32 {
    let normalized_screen = 1.0 / f32(screen_dimension);
    let normalized_map = map_pos * normalized_screen;
    let normalized_tile = tile_size * normalized_screen;

    let vertex = f32(tile_pos) * normalized_tile + select(0.0, normalized_tile, extend_to_end);
    let normalized_vertex = vertex * normalized_screen;

    return normalized_map + normalized_vertex;
}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let colour = colour_data_from_vec2(vertex.colour);

    let rgba_sample = vec4(colour.r, colour.g, colour.b, colour.a);
    let atlas_uv = _atlas_uv(vertex.atlas_item_index, colour.atlas_item_division_x, colour.atlas_item_division_y, vertex.corner);
    let atlas_sample = textureSample(atlas_diffuse, atlas_sampler, atlas_uv);
    let blended_sample = vec4(rgba_sample.rgb, atlas_sample.a);

    let sample = select(
        blended_sample,
        select(
            atlas_sample,
            select(
                rgba_sample,
                vec4(0.0),
                !colour.modulate_disabled
            ),
            !colour.atlas_disabled
        ),
        !colour.atlas_disabled && !colour.modulate_disabled
    );
    return sample;
}

fn _atlas_uv(atlas_item_index: u32, division_x: u32, division_y: u32, corner: u32) -> vec2<f32> {
    let corner_offset = array<vec2<f32>, 4>(
        vec2(0.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, 1.0),
        vec2(1.0, 1.0)
    )[corner];

    let atlas_item = atlas_items[atlas_item_index];
    let padding_wh = _atlas_to_uv_coords(vec2(atlas_padding, atlas_padding));

    let total_atlas_item_padding = padding_wh * vec2(f32(atlas_item.position.x + 1), f32(atlas_item.position.x + 1));
    let atlas_item_uv = _atlas_to_uv_coords(atlas_item.position) + total_atlas_item_padding;
    let atlas_item_wh = _atlas_to_uv_coords(atlas_item.size);

    let division_wh = atlas_item_wh / vec2(f32(atlas_item.divisions.x), f32(atlas_item.divisions.y));
    let division_uv = vec2<f32>(
        (division_wh.x * corner_offset.x) + (division_wh.x * f32(division_x)),
        (division_wh.y * corner_offset.y) + (division_wh.y * f32(division_y))
    );

    return division_uv;
}

fn _atlas_to_uv_coords(atlas_coords: vec2<u32>) -> vec2<f32> {
    let atlas_size = textureDimensions(atlas_diffuse);
    return vec2(
        f32(atlas_coords.x) / f32(atlas_size.x),
        f32(atlas_coords.y) / f32(atlas_size.y),
    );
}

struct ColourData {
    atlas_disabled: bool,
    modulate_disabled: bool,
    atlas_item_division_x: u32,
    atlas_item_division_y: u32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
};

/**
 * ColourData can be stored as a vec2<u32> with the following layout, where each section takes 8 bits
 * - metadata
 * - metadata
 * - atlas item division x
 * - atlas item division y
 * - red channel
 * - green channel
 * - blue channel
 * - alpha channel
 */
fn colour_data_from_vec2(input: vec2<u32>) -> ColourData {
    let metadata = slice_u32(input.x, 0u, 16u);
    let max_rgba_channel = 255.0;

    var colour: ColourData;
    colour.atlas_disabled = (metadata & 1u) != 0u;
    colour.modulate_disabled = ((metadata >> 1u) & 1u) != 0u;
    colour.atlas_item_division_x = slice_u32(input.x, 16u, 24u);
    colour.atlas_item_division_y = slice_u32(input.x, 24u, 32u);
    colour.r = f32(slice_u32(input.y, 0u, 8u)) / max_rgba_channel;
    colour.g = f32(slice_u32(input.y, 8u, 16u)) / max_rgba_channel;
    colour.b = f32(slice_u32(input.y, 16u, 24u)) / max_rgba_channel;
    colour.a = f32(slice_u32(input.y, 24u, 32u)) / max_rgba_channel;

    return colour;
}

fn slice_u32(value: u32, start: u32, end: u32) -> u32 {
    let length = 32u;

    let low_bits_filtered = value >> start;
    let high_bits_filtered = low_bits_filtered << (length - end);
    let bits_shifted_to_end = high_bits_filtered >> (length - end);

    return bits_shifted_to_end;
}
