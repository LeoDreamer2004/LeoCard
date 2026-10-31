import bevy_ui_render::ui_vertex_output::UiVertexOutput;

// 屏幕物理像素中的头像圆心、半径和整体遮罩不透明度。
@group(1) @binding(0)
var<uniform> spotlight: vec4<f32>;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let distance = length(in.position.xy - spotlight.xy);
    let outside = smoothstep(spotlight.z - 0.75, spotlight.z + 0.75, distance);
    return vec4<f32>(0.0, 0.0, 0.0, spotlight.w * outside);
}
