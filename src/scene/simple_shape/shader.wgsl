@group(0) @binding(0)
var texture_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;
@group(1) @binding(0)
var<uniform> camera: CameraInput;

struct CameraInput {
    view_projection: mat4x4<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_pos: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_pos: vec2<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var vertex_output: VertexOutput;
    vertex_output.clip_position = camera.view_projection * vec4<f32>(input.position, 1.0);
    vertex_output.texture_pos = input.texture_pos;
    return vertex_output;
}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_diffuse, texture_sampler, vertex.texture_pos);
}
