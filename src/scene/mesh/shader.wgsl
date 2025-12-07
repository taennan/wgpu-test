//
// Mesh
//

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var atlas_diffuse: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;

struct Camera {
    projection: mat4x4<f32>,
}

struct VertexInput {
    // Per vertex
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    // Per instance
    //@location(2) rotation_0: vec4<f32>,
    //@location(3) rotation_1: vec4<f32>,
    //@location(4) rotation_2: vec4<f32>,
    //@location(5) rotation_3: vec4<f32>,
    @location(2) texture_offset: vec2<f32>,
    @location(3) texture_size: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    //var rotation = mat4x4(input.rotation_0, input.rotation_1, input.rotation_2, input.rotation_3);
    out.clip_position = camera.projection * vec4(input.position, 1.0);
    out.uv = input.texture_offset + (input.uv / input.texture_size);
    return out;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return textureSample(atlas_diffuse, atlas_sampler, input.uv);
}
