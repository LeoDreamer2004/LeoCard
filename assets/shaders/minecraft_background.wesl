import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct BackgroundParams {
    // Spin in radians, transition progress, inverse UI scale, padding.
    params: vec4<f32>,
}

@group(1) @binding(0) var<uniform> material: BackgroundParams;
@group(1) @binding(1) var panorama: texture_cube<f32>;
@group(1) @binding(2) var panorama_sampler: sampler;
@group(1) @binding(3) var dirt: texture_2d<f32>;
@group(1) @binding(4) var dirt_sampler: sampler;

fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    return select(color / 12.92, pow((color + 0.055) / 1.055, vec3<f32>(2.4)), color > vec3<f32>(0.04045));
}

fn panorama_direction(uv: vec2<f32>, size: vec2<f32>, spin: f32) -> vec3<f32> {
    // Minecraft's 85-degree vertical FOV and Rx(180 + 10) * Ry(-spin).
    let aspect = size.x / max(size.y, 1.0);
    let eye = vec3<f32>((uv.x * 2.0 - 1.0) * aspect * 0.9163312, (1.0 - uv.y * 2.0) * 0.9163312, -1.0);
    let pitched = vec3<f32>(eye.x, -0.98480775 * eye.y - 0.17364818 * eye.z, 0.17364818 * eye.y - 0.98480775 * eye.z);
    let c = cos(spin);
    let s = sin(spin);
    return vec3<f32>(c * pitched.x + s * pitched.z, pitched.y, -s * pitched.x + c * pitched.z);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Classic menu dirt enlarged to 64 logical pixels per tile, tinted to 64/255.
    let dirt_uv = fract(in.uv * in.size * material.params.z / 64.0);
    let dirt_color = srgb_to_linear(textureSample(dirt, dirt_sampler, dirt_uv).rgb * (64.0 / 255.0));
    let direction = panorama_direction(in.uv, in.size, material.params.x);
    let panorama_color = textureSampleLevel(panorama, panorama_sampler, direction, 0.0).rgb;
    let blend = smoothstep(0.0, 1.0, material.params.y);
    return vec4<f32>(mix(dirt_color, panorama_color, blend), 1.0);
}
