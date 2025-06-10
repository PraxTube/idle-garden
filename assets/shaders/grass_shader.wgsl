#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_vertex_output::VertexOutput,
}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var noise_texture: texture_2d<f32>;
@group(2) @binding(3) var noise_sampler: sampler;
@group(2) @binding(4) var<uniform> texel_size: vec2<f32>;

fn to_grayscale(c: vec3<f32>) -> f32 {
    return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
}

fn from_grayscale(g: f32) -> vec3<f32> {
    return vec3(g, g, g);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let time_scale = 1.1;
    let distortion = 2.0;

    let snapped_mesh_uv = floor(mesh.uv / texel_size + 0.5) * texel_size;

    let original_color = textureSample(texture, texture_sampler, snapped_mesh_uv);

    let noise_color = textureSample(noise_texture, noise_sampler, fract(vec2(snapped_mesh_uv.y * 0.8, 0.0) + vec2(0.1, 0.1) * globals.time * time_scale));
    let raw_noise_offset = to_grayscale(noise_color.rgb);
    let noise_offset = (raw_noise_offset - 0.5) * original_color.r * distortion;
    let snapped_offset = (floor(noise_offset / texel_size + 0.5) * texel_size);
    // We scale by the texel size so that the distortion isn't relative to the size,
    // this means that a 1 pixel shift is the same on a 16x16 texture just as it would be on a 120x120 texture.
    let offset = snapped_offset * distortion * texel_size;

    let uv = fract(snapped_mesh_uv + vec2(offset.x, 0.0));

    let color = textureSample(texture, texture_sampler, uv);
    if color.g != 1.0 {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }

    return vec4(0.1, 0.8, 0.15, 1.0);
}
