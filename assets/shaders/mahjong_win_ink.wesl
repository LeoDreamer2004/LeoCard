import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct MahjongWinInkMaterial {
    animation: vec4<f32>,
}

@group(1) @binding(0) var<uniform> material: MahjongWinInkMaterial;
@group(1) @binding(1) var ink_texture: texture_2d<f32>;
@group(1) @binding(2) var ink_sampler: sampler;
@group(1) @binding(3) var progress_texture: texture_2d<f32>;
@group(1) @binding(4) var progress_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let ink = textureSample(ink_texture, ink_sampler, in.uv);
    let progress = textureSample(progress_texture, progress_sampler, in.uv).r;
    let point = in.uv * vec2<f32>(930.0, 275.0);
    let grain = sin(point.x * 0.17 + sin(point.y * 0.24) * 2.5) * 0.009
        + cos(point.y * 0.4 + point.x * 0.03) * 0.008;
    // The mask follows the curved brush gesture, including its curled leading edge.
    let painted = smoothstep(-0.0325, 0.0325, material.animation.x - progress + grain);
    let remaining = smoothstep(-0.06, 0.06, progress - material.animation.y + grain * 2.0);
    return vec4<f32>(ink.rgb, ink.a * painted * remaining * material.animation.z);
}
