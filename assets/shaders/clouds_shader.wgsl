#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_vertex_output::VertexOutput,
}

@group(2) @binding(0) var primary_texture: texture_2d<f32>;
@group(2) @binding(1) var primary_sampler: sampler;
@group(2) @binding(2) var secondary_texture: texture_2d<f32>;
@group(2) @binding(3) var secondary_sampler: sampler;
@group(2) @binding(4) var tertiary_texture: texture_2d<f32>;
@group(2) @binding(5) var tertiary_sampler: sampler;
@group(2) @binding(6) var quaternary_texture: texture_2d<f32>;
@group(2) @binding(7) var quaternary_sampler: sampler;
@group(2) @binding(8) var<uniform> texel_size: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let move_direction = vec2(-1.0, -0.7);
    let primary_offset = move_direction * globals.time * 0.01;

    let secondary_offset = vec2(1.0, 1.0) * -globals.time * 0.001;
    let tertiary_offset = vec2(1.0, -1.0) * globals.time * 0.03;
    let quaternary_offset = vec2(-1.0, 0.5) * globals.time * 0.0001;

    let snapped_mesh_uv = floor(fract(mesh.uv + primary_offset) / texel_size.xy + 0.5) * texel_size.xy;
    let primary_uv = fract(snapped_mesh_uv);
    let secondary_uv = fract(snapped_mesh_uv + secondary_offset);
    let tertiary_uv = fract(snapped_mesh_uv + tertiary_offset);
    let quaternary_uv = fract(snapped_mesh_uv + quaternary_offset);

    // We center the color at 0.0, meaning we shift the range from [0, 1.0] to [-0.5, 0.5].
    // That makes summing them up easier.
    let primary_noise = textureSample(primary_texture, primary_sampler, primary_uv).x - 0.5;
    let secondary_noise = textureSample(secondary_texture, secondary_sampler, secondary_uv).x - 0.5;
    let tertiary_noise = textureSample(tertiary_texture, tertiary_sampler, tertiary_uv).x - 0.5;
    let quaternary_noise = textureSample(quaternary_texture, quaternary_sampler, quaternary_uv).x - 0.5;

    let color = primary_noise + secondary_noise * 0.5 + tertiary_noise * 0.03 + quaternary_noise * 0.5;

    if color < 0.0 {
        return vec4(0.0, 0.0, 0.0, 0.3);
    }
    return vec4(0.0, 0.0, 0.0, 0.0);
}
