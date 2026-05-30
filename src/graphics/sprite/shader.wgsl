//
// Sprite Shader
//

@group(0) @binding(0) var<uniform> screen_height_aspect: f32;

@group(1) @binding(0) var<uniform> atlas_diffuse: texture_2d<f32>;
@group(1) @binding(1) var<uniform> atlas_sampler: sampler;

struct VertexInput {
    // Per vertex
    @location(0) xy: vec2<f32>,
    @location(1) uv: vec2<f32>,
    // Per instance
    @location(2) scale: vec2<f32>,
    @location(3) position: vec3<f32>,
    @location(4) texture_offset: vec2<f32>,
    @location(5) texture_size: vec2<f32>,
    @location(6) texture_divisions: vec2<u32>,
    @location(7) texture_division_coords: vec2<u32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let vertex_pos = vec3(
        input.xy.x * input.scale.x,
        input.xy.y * screen_height_aspect * input.scale.y,
        0.0
    ) + input.position;

    var out: VertexOutput;
    out.clip_position = vec4(vertex_pos, 1.0);
    out.uv = input.texture_offset + (input.uv / input.texture_size);
    return out;
}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_diffuse, texture_sampler, vertex.uv);
}
