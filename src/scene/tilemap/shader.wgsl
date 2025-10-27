//
// Tilemap Shader
//

@group(0) @binding(0)
var<uniform> camera: Camera2DInput;
@group(1) @binding(0)
var texture_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

struct Camera2DInput {
    position: vec3<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var vertex_output: VertexOutput;
    vertex_output.position = vec4<f32>(input.position - camera.position, 1.0);
    vertex_output.uv = input.uv;
    return vertex_output;
}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_diffuse, texture_sampler, vertex.uv);
}
