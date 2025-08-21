#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_functions as mesh_functions,
}

const AMPLITUDE: f32 = 12.0;
const TIME_SCALE_SINE: f32 = 1.0;
const TIME_SCALE_EXP: f32 = 0.4;

@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;
@group(2) @binding(3) var sine_texture: texture_2d<f32>;
@group(2) @binding(4) var sine_sampler: sampler;
@group(2) @binding(5) var exp_texture: texture_2d<f32>;
@group(2) @binding(6) var exp_sampler: sampler;
@group(2) @binding(7) var timestamps_texture: texture_2d<f32>;
@group(2) @binding(8) var timestamps_texture_sampler: sampler;

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
    let timestamp_dim = textureDimensions(timestamps_texture);
    let x = mesh.index % timestamp_dim.x;
    let y = mesh.index / timestamp_dim.x;

    let sine_timestamp_uv = vec2<f32>(f32(x) / f32(timestamp_dim.x), f32(y) / f32(timestamp_dim.y));
    let sine_raw = textureSample(timestamps_texture, timestamps_texture_sampler, sine_timestamp_uv).x;
    let sine_timestamp = abs(sine_raw);
    let sine_sign = sign(sine_raw);

    let exp_timestamp_uv = vec2<f32>(sine_timestamp_uv.x, sine_timestamp_uv.y + 0.5);
    let exp_timestamp = textureSample(timestamps_texture, timestamps_texture_sampler, exp_timestamp_uv).x;

    let exp_t = (globals.time - exp_timestamp) * TIME_SCALE_EXP;
    if exp_t > 1.0 {
        return textureSample(texture, texture_sampler, mesh.uv);
    }
    let exp = textureSample(exp_texture, exp_sampler, vec2(exp_t, 0.0)).x;

    if exp <= 0.1 {
        return textureSample(texture, texture_sampler, mesh.uv);
    }

    let texel_size = 1.0 / vec2<f32>(textureDimensions(texture));
    let snapped_mesh_uv = (floor(mesh.uv / texel_size) + 0.5) * texel_size;

    let sine_t = fract((globals.time - sine_timestamp) * TIME_SCALE_SINE);
    let raw_sine = textureSample(sine_texture, sine_sampler, vec2(sine_t, 0.0)).x;
    let sine = sine_sign * (raw_sine - 0.5) * 2.0 * (1.0 - snapped_mesh_uv.y - texel_size.y);

    let noise_offset = exp * AMPLITUDE * sine;
    let snapped_offset = (floor(noise_offset / texel_size.x) + 0.5) * texel_size.x;
    // We scale by the texel size so that the distortion isn't relative to the size,
    // this means that a 1 pixel shift is the same on a 16x16 texture just as it would be on a 120x120 texture.
    let offset = snapped_offset * texel_size.x;

    let uv = fract(snapped_mesh_uv + vec2(offset, 0.0));
    return textureSample(texture, texture_sampler, uv);
}
