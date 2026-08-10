#import bevy_ui::ui_vertex_output::UiVertexOutput

struct TexasChipZoneMaterial {
    // Column 0: inner brightness, border brightness, border width px, radius px.
    // Column 1: zone left, zone top, table width, table height.
    // Column 2 x: whether the table texture is tiled.
    params: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> material: TexasChipZoneMaterial;
@group(1) @binding(1)
var felt_texture: texture_2d<f32>;
@group(1) @binding(2)
var felt_sampler: sampler;

fn rounded_rectangle_distance(point: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(point) - (half_size - vec2<f32>(radius));
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = max(in.size, vec2<f32>(1.0));
    let appearance = material.params[0];
    let placement = material.params[1];
    let mapping = material.params[2];
    let radius = min(appearance.w, min(size.x, size.y) * 0.5);
    let point = (in.uv - vec2<f32>(0.5)) * size;
    let signed_distance = rounded_rectangle_distance(point, size * 0.5, radius);
    let inside_distance = -signed_distance;

    let border_width = max(appearance.z, 1.0);
    let border = 1.0 - smoothstep(border_width, border_width + 1.0, inside_distance);
    let brightness = mix(appearance.x, appearance.y, border);
    let shape_alpha = 1.0 - smoothstep(-0.5, 0.75, signed_distance);

    let table_pixel = placement.xy + in.uv * size;
    let texture_size = vec2<f32>(textureDimensions(felt_texture));
    var felt_uv: vec2<f32>;
    if mapping.x > 0.5 {
        felt_uv = fract(table_pixel / max(texture_size, vec2<f32>(1.0)));
    } else {
        felt_uv = clamp(table_pixel / max(placement.zw, vec2<f32>(1.0)), vec2<f32>(0.0), vec2<f32>(1.0));
    }
    let felt = textureSample(felt_texture, felt_sampler, felt_uv);

    return vec4<f32>(felt.rgb * brightness, felt.a * shape_alpha);
}
