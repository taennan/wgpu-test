//
// Tilemap Shader v2
//

@group(0) @binding(0)
var<uniform> camera: Camera2DInput;

@group(1) @binding(0)
var texture_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;
@group(1) @binding(2)
var texture_tilecount: vec2<u32>;
@group(1) @binding(3)
var tile_size: vec2<f32>;

const left_corner = 0u;
const right_corner = 1u;
const top_corner = 2u;
const bottom_corner = 3u;

struct Camera2DInput {
    position: vec3<f32>,
};

struct VertexInput {
    @location(0) tile_position: vec3<u32>,
    @location(1) tile_texture_index: u32,
    @location(2) corner: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var corner_position = compute_corner_position(input.tile_position, input.corner);

    var vertex_output: VertexOutput;
    vertex_output.position = vec4<f32>(corner_position - camera.position, 1.0);
    vertex_output.uv = compute_corner_uv(input.tile_texture_index, input.corner);
    return vertex_output;
}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_diffuse, texture_sampler, vertex.uv);
}

fn compute_corner_position(tile_position: vec3<u32>, corner: u32) -> vec3<f32> {
    let half_tile_x = tile_size.x * 0.5;
    let half_tile_y = tile_size.y * 0.5;

    var offset_x = inverse_f32_if(half_tile_x, corner == left_corner);
    var offset_y = inverse_f32_if(half_tile_y, corner == top_corner);

    let x = offset_x + tile_size.x * f32(tile_position.x);
    let y = offset_y + tile_size.y * f32(tile_position.y);
    let z = f32(tile_position.y);

    let position = vec3<f32>(x, y, z);
    return position;
}

fn inverse_f32_if(value: f32, condition: bool) -> f32 {
    if condition {
        return -value;
    }
    return value;
}

fn compute_corner_uv(tile_texture_index: u32, corner: u32) -> vec2<f32> {
    let half_tile_u = 1.0 / f32(texture_tilecount.x);
    let half_tile_v = 1.0 / f32(texture_tilecount.y);

    let offset_u = inverse_f32_if(half_tile_u, corner == left_corner);
    let offset_v = inverse_f32_if(half_tile_v, corner == top_corner);

    let texture_coords = texture_index_to_texture_coords(tile_texture_index);
    let u = offset_u + texture_coords.x * tile_size.x;
    let v = offset_v + texture_coords.y * tile_size.y;

    let uv = vec2<f32>(offset_u + u, offset_v + v);
    return uv;
}

fn texture_index_to_texture_coords(tile_texture_index: u32) -> vec2<f32> {
    let tile_u = tile_texture_index % texture_tilecount.x;
    let tile_v = tile_texture_index / texture_tilecount.x;

    let u = f32(tile_u) / f32(texture_tilecount.x);
    let v = f32(tile_v) / f32(texture_tilecount.y);

    let uv = vec2<f32>(u, v);
    return uv;
}
