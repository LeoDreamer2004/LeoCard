import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct BackgroundMaterial {
    animation: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: BackgroundMaterial;

fn ribbon(
    uv: vec2<f32>,
    seconds: f32,
    slope: f32,
    offset: f32,
    width: f32,
    phase: f32,
) -> f32 {
    let flow = seconds * 0.024 + phase;
    let curve = offset + slope * (uv.x - 0.5)
        + 0.10 * sin(uv.x * 4.0 + flow)
        + 0.035 * sin(uv.x * 7.0 - flow * 0.67);
    let breathing_width = width * (1.0 + 0.09 * sin(seconds * 0.31 + phase));
    let distance = (uv.y - curve) / breathing_width;
    let halo = exp(-distance * distance * 0.55);
    let body = smoothstep(-1.1, -0.55, distance) * (1.0 - smoothstep(0.20, 1.15, distance));
    let edge_distance = distance + 0.70;
    let edge = exp(-edge_distance * edge_distance * 36.0);
    let travel = 0.82 + 0.18 * sin(uv.x * 3.4 - seconds * 0.065 + phase);
    let breath = 0.86 + 0.14 * sin(seconds * 0.43 + phase);
    return (halo * 0.20 + body * 0.65 + edge * 0.18) * travel * breath;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let seconds = material.animation.x;
    // Colors are linear, keeping the base dark and the ribbons softly luminous.
    var color = mix(vec3<f32>(0.008, 0.007, 0.019), vec3<f32>(0.014, 0.011, 0.029), uv.x);
    let upper = ribbon(uv, seconds, 1.25, -0.10, 0.11, 0.4);
    let lower = ribbon(uv, seconds, -0.34, 0.93, 0.13, 2.3);
    let crossing = ribbon(uv, seconds, 0.50, 0.76, 0.20, 4.6);
    color += upper * vec3<f32>(0.075, 0.055, 0.19);
    color += lower * vec3<f32>(0.070, 0.027, 0.14);
    color += crossing * vec3<f32>(0.013, 0.033, 0.057) * 0.55;

    let aspect = in.size.x / max(in.size.y, 1.0);
    let centered = (uv - vec2<f32>(0.5)) * vec2<f32>(aspect, 1.0);
    let vignette = smoothstep(0.30, 1.10, length(centered));
    color *= 1.0 - 0.32 * vignette;

    // A stationary fine dither softens gradients without an animated noise shimmer.
    let grain = fract(sin(dot(floor(in.position.xy), vec2<f32>(12.9898, 78.233))) * 43758.5453);
    color += vec3<f32>((grain - 0.5) * 0.0007);
    return vec4<f32>(max(color, vec3<f32>(0.0)), 1.0);
}
