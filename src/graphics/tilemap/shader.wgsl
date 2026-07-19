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
    /*
     * This field encodes either the rgba colour of the tile, or the atlas data
     * It will be in the following form if encoding atlas data:
     * - 0bxxx (unused)
     * - 0bxxx (atlas item division x)
     * - 0bxxx (atlas item division y)
     * - 0bxx1 (is atlas item if least significant bit is 1)
     * It will be the following if raw rgba data, with all items capped at 255 in decimal:
     * - 0bxxx (r)
     * - 0bxxx (g)
     * - 0bxxx (b)
     * - 0bxx0 (a, is rgba data if most significant bit is 0)
     *
     * We could probably compress this even more by packing 2 u16's into a single channel
     * But that should only be done a lot later...
     */
    @location(2) colour: vec4<u32>,
    // Per instance
    @location(3) map_pos: vec3<f32>,
    @location(4) tile_size: vec2<f32>,
    @location(5) atlas_item_index: u32,

};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) colour: vec4<u32>,
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
    let rgba_sample = _rgba_sample(vertex.colour);
    let atlas_uv = _atlas_uv(vertex.atlas_item_index, vertex.colour.g, vertex.colour.b, vertex.corner);
    let atlas_sample = textureSample(atlas_diffuse, atlas_sampler, atlas_uv);

    let is_atlas_data = (vertex.colour.a & 1) > 0;
    let sample = select(rgba_sample, atlas_sample, is_atlas_data);
    return sample;
}

fn _rgba_sample(rgba: vec4<u32>) -> vec4<f32> {
    return vec4(
        _u32_to_f32_colour_channel(rgba.r),
        _u32_to_f32_colour_channel(rgba.g),
        _u32_to_f32_colour_channel(rgba.b),
        _u32_to_f32_colour_channel(rgba.a << 1)
    );
}

fn _u32_to_f32_colour_channel(channel: u32) -> f32 {
    return f32(channel) / 255.0;
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
