#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_vertex_output::VertexOutput,
}

const AMPLITUDE: f32 = 12.0;
const TIME_SCALE_SINE: f32 = 1.0;
const TIME_SCALE_EXP: f32 = 0.4;

// First vec2 is texel_size, third f32 is sine timestamp, last element is exp damp timestamp.
@group(2) @binding(0) var<uniform> texel_size_and_timestamps: vec4<f32>;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;
@group(2) @binding(3) var sine_texture: texture_2d<f32>;
@group(2) @binding(4) var sine_sampler: sampler;
@group(2) @binding(5) var exp_texture: texture_2d<f32>;
@group(2) @binding(6) var exp_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let texel_size = texel_size_and_timestamps.xy;
    let sine_timestamp = abs(texel_size_and_timestamps.z);
    let sine_sign = sign(texel_size_and_timestamps.z);
    let exp_timestamp = texel_size_and_timestamps.w;
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
