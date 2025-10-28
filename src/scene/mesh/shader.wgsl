//
// Mesh
//

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var texture_diffuse: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

struct Camera {
    projection: mat4x4<f32>,
}

struct VertexInput {
    // Per vertex
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    // Per instance
    @location(2) rotation: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    return out;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_diffuse, texture_sampler, input.uv);
}
