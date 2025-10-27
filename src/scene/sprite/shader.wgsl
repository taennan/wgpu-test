//
// Sprite Shader
//

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var texture_diffuse: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;
//@group(1) @binding(2) var<uniform> texture_divisions: vec2<u32>;

struct Camera {
    projection: mat4x4<f32>,
};

struct VertexInput {
    //@builtin(vertex_index) vertex_index: u32,
    @location(0) vertex_index: u32,
    @location(1) size: vec2<f32>,
    @location(2) position: vec3<f32>,
    //@location(3) texture_division_coords: vec2<u32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = get_vertex_position(input);
    output.uv = get_vertex_uv(input);
    return output;
}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_diffuse, texture_sampler, vertex.uv);
}

fn get_vertex_position(input: VertexInput) -> vec4<f32> {
    let corner_top_left = vec2(-1.0, 1.0);
    let corner_top_right = vec2(1.0, 1.0);
    let corner_bottom_left = vec2(-1.0, -1.0);
    let corner_bottom_right = vec2(1.0, -1.0);

    var xy_pos = vec2(0.0, 0.0);
    if input.vertex_index == 0u || input.vertex_index == 3u {
        xy_pos = corner_bottom_left;
    } else if input.vertex_index == 1 || input.vertex_index == 5 {
        xy_pos = corner_top_right;
    } else if input.vertex_index == 2 {
        xy_pos = corner_top_left;
    } else if input.vertex_index == 4 {
        xy_pos = corner_bottom_right;
    }
    //xy_pos *= input.size;

    //let xyz_position = input.position + vec3(xy_pos.x * input.position.x, xy_pos.y * input.position.y, input.position.z);
    //let in_camera_position = camera.projection * vec4(xyz_position, 1.0);

    let xyz_position = input.position + vec3(xy_pos.x, xy_pos.y, 0.0);
    let in_camera_position = vec4(xyz_position, 1.0);

    return in_camera_position;
}

fn get_vertex_uv(input: VertexInput) -> vec2<f32> {
    let corner_top_left = vec2(0.0, 0.0);
    let corner_top_right = vec2(1.0, 0.0);
    let corner_bottom_left = vec2(0.0, 1.0);
    let corner_bottom_right = vec2(1.0, 1.0);

    var division_uv = vec2(0.0, 0.0);
    if input.vertex_index == 0u || input.vertex_index == 3u {
        division_uv = corner_bottom_left;
    } else if input.vertex_index == 1 || input.vertex_index == 5 {
        division_uv = corner_top_right;
    } else if input.vertex_index == 2 {
        division_uv = corner_top_left;
    } else if input.vertex_index == 4 {
        division_uv = corner_bottom_right;
    }
    return division_uv;
    //division_uv = division_uv / vec2<f32>(texture_divisions);

    //let uv = division_uv * vec2<f32>(input.texture_division_coords);
    //return uv;
}
