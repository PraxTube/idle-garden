#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_functions as mesh_functions,
}

const AMPLITUDE: f32 = 12.0;
const TIME_SCALE_SINE: f32 = 1.0;
const TIME_SCALE_EXP: f32 = 0.4;

// First element is sine timestamp, second element is exp timestamp.
// Padded to 16 bytes for WASM (note sure if that is actually required here, but just to be safe).
@group(2) @binding(0) var<storage, read> timestamps: array<vec4<f32>, 16384>;

@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;
@group(2) @binding(3) var sine_texture: texture_2d<f32>;
@group(2) @binding(4) var sine_sampler: sampler;
@group(2) @binding(5) var exp_texture: texture_2d<f32>;
@group(2) @binding(6) var exp_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) index: u32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    out.uv = vertex.uv;

    let tag = mesh_functions::get_tag(vertex.instance_index);
    out.index = tag;

    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0)
    );
    out.position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);

    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / vec2<f32>(textureDimensions(texture));
    let sine_timestamp = abs(timestamps[mesh.index % 16384].x);
    let sine_sign = sign(timestamps[mesh.index].x);
    let exp_timestamp = timestamps[mesh.index].y;
    let snapped_mesh_uv = floor(mesh.uv / texel_size) * texel_size;

    let sine_t = fract((globals.time - sine_timestamp) * TIME_SCALE_SINE);
    let raw_sine = textureSample(sine_texture, sine_sampler, vec2(sine_t, 0.0)).x;
    let sine = sine_sign * (raw_sine - 0.5) * 2.0 * (1.0 - snapped_mesh_uv.y - texel_size.y);

    let exp_t = (globals.time - exp_timestamp) * TIME_SCALE_EXP;
    if exp_t > 1.0 {
        return textureSample(texture, texture_sampler, mesh.uv);
    }
    let exp = textureSample(exp_texture, exp_sampler, vec2(exp_t, 0.0)).x;


    let noise_offset = exp * AMPLITUDE * sine;
    let snapped_offset = floor(noise_offset / texel_size) * texel_size;
    // We scale by the texel size so that the distortion isn't relative to the size,
    // this means that a 1 pixel shift is the same on a 16x16 texture just as it would be on a 120x120 texture.
    let offset = snapped_offset * texel_size;

    let uv = fract(snapped_mesh_uv + vec2(offset.x, 0.0));
    return textureSample(texture, texture_sampler, uv);
}
