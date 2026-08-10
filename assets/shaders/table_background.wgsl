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

fn normalized_uv(uv: vec2<f32>, tiled: bool) -> vec2<f32> {
    if tiled {
        return fract(uv);
    }
    return clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
}

fn table_sample(uv: vec2<f32>, tiled: bool) -> vec4<f32> {
    return textureSample(table_texture, table_sampler, normalized_uv(uv, tiled));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let tiled = material.params.z > 0.5;
    let dimensions = vec2<f32>(textureDimensions(table_texture));
    var uv = in.uv;
    if tiled {
        let tile_scale = in.size / max(dimensions, vec2<f32>(1.0));
        uv *= tile_scale;
    }
    let color = table_sample(uv, tiled);

    let centered = abs(in.uv * 2.0 - vec2<f32>(1.0));
    let edge = smoothstep(0.30, 1.0, max(centered.x, centered.y));
    let shaded_rgb = color.rgb * material.params.y * (1.0 - edge * material.params.x);
    return vec4<f32>(shaded_rgb, 1.0);
}
