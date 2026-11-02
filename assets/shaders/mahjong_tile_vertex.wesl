import bevy_render::view::View;
import bevy_ui_render::ui_vertex_output::UiVertexOutput;
import super::mahjong_tile_bindings::material;
@group(0) @binding(0) var<uniform> view: View;

@vertex
fn vertex(
    @location(0) position: vec3<f32>, @location(1) uv: vec2<f32>,
    @location(2) size: vec2<f32>, @location(3) borders: vec4<f32>,
    @location(4) radius_x: vec4<f32>, @location(5) radius_y: vec4<f32>,
) -> UiVertexOutput {
    var out: UiVertexOutput;
    var pixel = position;
    if material.ray_origin.w > 0.0 {
        pixel = vec3<f32>(material.paint_bounds.xy + uv * material.paint_bounds.zw, position.z);
    }
    out.position = view.clip_from_world * vec4<f32>(pixel, 1.0);
    out.uv = uv; out.size = size; out.border_widths = borders;
    out.border_radius_x = radius_x; out.border_radius_y = radius_y;
    return out;
}
