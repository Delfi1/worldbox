#import bevy_pbr::{
    forward_io::{FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_normal_local_to_world}
#import bevy_pbr::pbr_functions::{calculate_view, prepare_world_normal}
#import bevy_pbr::mesh_bindings::mesh
#import bevy_pbr::pbr_types::pbr_input_new
#import bevy_pbr::prepass_utils
#import bevy_pbr::mesh_view_bindings::view
#import bevy_core_pipeline::tonemapping::tone_mapping

@group(2) @binding(0) var textures: binding_array<texture_2d_array<f32>>;
@group(2) @binding(1) var nearest_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) data: u32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) b: u32,
    @location(4) side: u32,
    @location(5) instance_index: u32,
};

var<private> normals: array<vec3<f32>, 6> = array<vec3<f32>,6> (
	vec3<f32>(0.0, 1.0, 0.0),   // Up
    vec3<f32>(-1.0, 0.0, 0.0),  // Left
	vec3<f32>(1.0, 0.0, 0.0),   // Right
	vec3<f32>(0.0, 0.0, -1.0),  // Forward
	vec3<f32>(0.0, 0.0, 1.0),   // Back
    vec3<f32>(0.0, -1.0, 0.0),  // Down
);

fn x_bits(bits: u32) -> u32{
    return (1u << bits) - 1u;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let x = f32(vertex.data & x_bits(6u));
    let y = f32(vertex.data >> 6u & x_bits(6u));
    let z = f32(vertex.data >> 12u & x_bits(6u));
    let normal_index = vertex.data >> 18u & x_bits(3u);
    let b = vertex.data >> 21u & x_bits(7u);
    let uvx = vertex.data >> 28u & x_bits(1u);
    let uvy = vertex.data >> 29u & x_bits(1u);

    let local_position = vec4<f32>(x, y, z, 1.0);
    let world_position = get_world_from_local(vertex.instance_index) * local_position;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(vertex.instance_index),
        local_position,
    );

    out.world_position = world_position;
    let normal = normals[normal_index];
    out.world_normal = mesh_normal_local_to_world(normal, vertex.instance_index);
    out.uv = vec2(f32(uvx), f32(uvy));
    out.b = b;
    out.side = normal_index;
    out.instance_index = vertex.instance_index;
    return out;
}

@fragment
fn fragment(input: VertexOutput) -> FragmentOutput {
    var pbr_input = pbr_input_new();

    pbr_input.flags = mesh[input.instance_index].flags;
    pbr_input.material.base_color = textureSample(textures[input.b], nearest_sampler, input.uv, input.side);

    pbr_input.V = calculate_view(input.world_position, pbr_input.is_orthographic);
    pbr_input.frag_coord = input.clip_position;
    pbr_input.world_position = input.world_position;

    pbr_input.world_normal = prepare_world_normal(
        input.world_normal,
        false,
        false,
    );

    pbr_input.material.metallic = 0.0;
    pbr_input.material.perceptual_roughness = 1.0;
    pbr_input.material.reflectance = 0.0;

    pbr_input.N = normalize(pbr_input.world_normal);

    var out: FragmentOutput;

    out.color = tone_mapping(apply_pbr_lighting(pbr_input), view.color_grading);
    return out;
}