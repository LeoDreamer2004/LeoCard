#import bevy_ui::ui_vertex_output::UiVertexOutput

struct TableBackgroundMaterial {
    // x: vignette strength, y: brightness, z: tiled flag, w: unused.
    params: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: TableBackgroundMaterial;
@group(1) @binding(1)
var table_texture: texture_2d<f32>;
@group(1) @binding(2)
var table_sampler: sampler;

fn cover_uv(
    uv: vec2<f32>,
    target_size: vec2<f32>,
    texture_size: vec2<f32>,
) -> vec2<f32> {
    let safe_target = max(target_size, vec2<f32>(1.0));
    let safe_texture = max(texture_size, vec2<f32>(1.0));
    let cover_scale = max(safe_target.x / safe_texture.x, safe_target.y / safe_texture.y);
    let visible_fraction = safe_target / (safe_texture * cover_scale);
    let crop_origin = (vec2<f32>(1.0) - visible_fraction) * 0.5;
    return crop_origin + uv * visible_fraction;
}

fn table_uv(
    uv: vec2<f32>,
    target_size: vec2<f32>,
    texture_size: vec2<f32>,
    tiled: bool,
) -> vec2<f32> {
    if tiled {
        return fract(uv * target_size / max(texture_size, vec2<f32>(1.0)));
    }
    return cover_uv(uv, target_size, texture_size);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(table_texture));
    let uv = table_uv(in.uv, in.size, dimensions, material.params.z > 0.5);
    let color = textureSample(table_texture, table_sampler, uv);

    let centered = abs(in.uv * 2.0 - vec2<f32>(1.0));
    let edge = smoothstep(0.30, 1.0, max(centered.x, centered.y));
    let shaded_rgb = color.rgb * material.params.y * (1.0 - edge * material.params.x);
    return vec4<f32>(shaded_rgb, 1.0);
}
