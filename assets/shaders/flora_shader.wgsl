#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_vertex_output::VertexOutput,
}

// vec2 is sufficient here, but we pad with 2 more f32 to have 16 byte aligned for WASM
@group(2) @binding(0) var<uniform> texel_size: vec4<f32>;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

fn noise_permute_vec3f(x: vec3<f32>) -> vec3<f32> {
    return (((x * 34.0) + 10.0) * x) % 289.0;
}

fn noise_worley(v: vec2<f32>, angle: f32) -> f32 {
    let k = 0.142857142857;     // 1/7
    let ko = 0.428571428571;    // 3/7
    let jitter = 1.0;           // Less gives more regular pattern

    let pi = floor(v) % 289.0;
    let pf = fract(v);
    let oi = vec3(-1.0, 0.0, 1.0);
    let of_ = vec3(-0.5, 0.5, 1.5);

    let px = noise_permute_vec3f(pi.x + oi);
    var p = noise_permute_vec3f(px.x + pi.y + oi);  // p11, p12, p13

    let s = sin(angle);
    let c = cos(angle);

    var ox = fract(p * k) - ko;
    var oy = (floor(p * k) % 7.0) * k - ko;
    var rot_ox = c * ox - s * oy;
    var rot_oy = s * ox + c * oy;

    var dx = pf.x + 0.5 + jitter * rot_ox;
    var dy = pf.y - of_ + jitter * rot_oy;
    var d1 = dx * dx + dy * dy;     // d11, d12 and d13, squared

    p = noise_permute_vec3f(px.y + pi.y + oi);      // p21, p22, p23
    ox = fract(p * k) - ko;
    oy = (floor(p * k) % 7.0) * k - ko;
    rot_ox = c * ox - s * oy;
    rot_oy = s * ox + c * oy;

    dx = pf.x - 0.5 + jitter * rot_ox;
    dy = pf.y - of_ + jitter * rot_oy;

    var d2 = dx * dx + dy * dy;     // d21, d22 and d23, squared
    p = noise_permute_vec3f(px.z + pi.y + oi);      // p31, p32, p33
    ox = fract(p * k) - ko;
    oy = (floor(p * k) % 7.0) * k - ko;
    rot_ox = c * ox - s * oy;
    rot_oy = s * ox + c * oy;

    dx = pf.x - 1.5 + jitter * rot_ox;
    dy = pf.y - of_ + jitter * rot_oy;
    let d3 = dx * dx + dy * dy;     // d31, d32 and d33, squared
	
    // Sort out the two smallest distances (F1, F2)
    d1 = min(d1, d2);
    d1 = min(d1, d3);
    d1.x = min(d1.x, d1.y);
    d1.x = min(d1.x, d1.z);

    return sqrt(d1.x);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let time_scale_first = 0.0001;
    let distortion = 2.5;

    let snapped_mesh_uv = floor(mesh.uv / texel_size.xy + 0.5) * texel_size.xy;

    let noise_offset_x = noise_worley(snapped_mesh_uv * 6.0, globals.time);
    let noise_offset_y = noise_worley(snapped_mesh_uv * 6.0, -globals.time * 0.8);
    let noise_offset = vec2(noise_offset_x, noise_offset_y);
    let snapped_offset = (floor(noise_offset / texel_size.xy + 0.5) * texel_size.xy) * distortion * texel_size.xy;

    let uv = fract(snapped_mesh_uv + snapped_offset);

    return textureSample(texture, texture_sampler, uv);
}
