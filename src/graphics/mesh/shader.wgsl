//
// Mesh
//

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var atlas_diffuse: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;
@group(1) @binding(2) var<uniform> atlas_padding: u32;
@group(1) @binding(3) var<storage, read> atlas_items: array<AtlasItem>;

struct Camera {
    projection: mat4x4<f32>,
}

struct AtlasItem {
    position: vec2<u32>,
    size: vec2<u32>,
    // X and Y must not be zero!!
    divisions: vec2<u32>,
}

struct VertexInput {
    // Per vertex
    @location(0) xyz: vec3<f32>,
    @location(1) uv: vec2<f32>,
    // Per instance
    //@location(2) rotation_0: vec4<f32>,
    //@location(3) rotation_1: vec4<f32>,
    //@location(4) rotation_2: vec4<f32>,
    //@location(5) rotation_3: vec4<f32>,
    @location(2) position: vec3<f32>,
    @location(3) atlas_item_index: u32,
    @location(4) texture_division_coords: vec2<u32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    //var rotation = mat4x4(input.rotation_0, input.rotation_1, input.rotation_2, input.rotation_3);
    out.position = camera.projection * vec4(input.xyz + input.position, 1.0);
    out.uv = _vertex_uv(input);
    return out;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(atlas_diffuse, atlas_sampler, input.uv);
}

fn _vertex_uv(input: VertexInput) -> vec2<f32> {
    let atlas_item = atlas_items[input.atlas_item_index];
    let atlas_item_uv = _atlas_to_uv_coords(atlas_item.position) + _atlas_to_uv_coords(vec2(atlas_padding, atlas_padding));
    let padding_uv = _atlas_to_uv_coords(vec2(atlas_padding, atlas_padding));
    let atlas_item_wh = _atlas_to_uv_coords(atlas_item.size) - padding_uv * 2.0;
    let atlas_item_division_wh = atlas_item_wh / vec2(f32(atlas_item.divisions.x), f32(atlas_item.divisions.y));

    let vertex_division_offset = vec2(atlas_item_division_wh.x * f32(input.texture_division_coords.x), atlas_item_division_wh.y * f32(input.texture_division_coords.y));
    let vertex_uv_inside_division = input.uv * atlas_item_division_wh;

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
