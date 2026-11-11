import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct EntranceVeil {
    screen_to_avatar: vec4<f32>,
    center: vec2<f32>,
    half_size: vec2<f32>,
    corner_radius: f32,
    opacity: f32,
}

@group(1) @binding(0)
var<uniform> veil: EntranceVeil;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    if veil.half_size.x <= 0.0 {
        return vec4<f32>(0.0, 0.0, 0.0, veil.opacity);
    }
    let to_avatar = mat2x2<f32>(veil.screen_to_avatar.xy, veil.screen_to_avatar.zw);
    let point = to_avatar * (in.position.xy - veil.center);
    let q = abs(point) - (veil.half_size - vec2<f32>(veil.corner_radius));
    let distance = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - veil.corner_radius;
    let edge = max(0.5 * fwidth(distance), 0.0001);
    let outside = smoothstep(-edge, edge, distance);
    return vec4<f32>(0.0, 0.0, 0.0, veil.opacity * outside);
}
