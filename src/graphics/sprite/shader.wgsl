//
// Sprite Shader
//

//
// NOTES:
// - UV coords in WebGPU are from 0 to 1 and start in the top left corner
//

@group(0) @binding(0) var<uniform> screen_size: vec2<u32>;

@group(1) @binding(0) var atlas_diffuse: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;
@group(1) @binding(2) var<uniform> atlas_padding: u32;
@group(1) @binding(3) var<storage, read> atlas_items: array<AtlasItem>;

struct AtlasItem {
    @location(0) position: vec2<u32>,
    @location(1) size: vec2<u32>,
    // X and Y must not be zero!!
    @location(2) divisions: vec2<u32>,
}

struct VertexInput {
    // Per vertex
    @location(0) xyz: vec3<f32>,
    @location(1) uv: vec2<f32>,
    // Per instance
    @location(2) position: vec3<f32>,
    @location(3) atlas_item_index: u32,
    @location(4) texture_division_coords: vec2<u32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = _vertex_uv(input);
    out.position = vec4(
        _vertex_position(input.xyz.x, input.position.x, screen_size.x),
        _vertex_position(input.xyz.y, input.position.y, screen_size.y),
        input.xyz.z,
        1.0
    );
    return out;
}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(atlas_diffuse, atlas_sampler, vertex.uv);
}

fn _vertex_position(vertex: f32, instance: f32, screen_dimension: u32) -> f32 {
    let normalized_screen = 1.0 / f32(screen_dimension);
    let normalized_instance = instance * normalized_screen;
    let normalized_vertex = vertex * normalized_screen;
    return normalized_vertex + normalized_instance;
}

fn _vertex_uv(input: VertexInput) -> vec2<f32> {
    let atlas_item = atlas_items[input.atlas_item_index];
    let atlas_item_uv = _atlas_to_uv_coords(atlas_item.position) + _atlas_to_uv_coords(vec2(atlas_padding, atlas_padding));
    let atlas_item_wh = _atlas_to_uv_coords(atlas_item.size);
    let atlas_item_division_wh = atlas_item_wh / vec2(f32(atlas_item.divisions.x), f32(atlas_item.divisions.y));

    let vertex_division_offset = vec2(atlas_item_division_wh.x * f32(input.texture_division_coords.x), atlas_item_division_wh.y * f32(input.texture_division_coords.y));
    let vertex_uv_inside_division = input.uv / atlas_item_division_wh;

    let vertex_uv = atlas_item_uv + vertex_division_offset + vertex_uv_inside_division;
    return vertex_uv;
}

fn _atlas_to_uv_coords(atlas_coords: vec2<u32>) -> vec2<f32> {
    let atlas_size = textureDimensions(atlas_diffuse);
    return vec2(
        f32(atlas_coords.x) / f32(atlas_size.x),
        f32(atlas_coords.y) / f32(atlas_size.y),
    );
}
