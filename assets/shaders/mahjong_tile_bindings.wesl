struct MahjongTileMaterial {
    // x: interaction; y: face/back; z: opacity.
    // w: -6 own hand, -5 concealed meld, -4 meld, -3 opposite stand, -2 side stand, -1 showcase, 2 double wall, 3 single wall.
    params: vec4<f32>,
    // xy: local light direction; z: matching-tile overlay; w: actual screen rotation.
    lighting: vec4<f32>,
    // rgb: state overlay color; w: strength. State overlays take precedence over matching tiles.
    highlight: vec4<f32>,
    elevation: f32,
    shadow_only: f32,
    paint_bounds: vec4<f32>,
    ray_origin: vec4<f32>,
    ray_forward: vec4<f32>,
    ray_right: vec4<f32>,
    ray_down: vec4<f32>,
    box_half_size: vec4<f32>,
    response: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: MahjongTileMaterial;
@group(1) @binding(1)
var glyph_texture: texture_2d<f32>;
@group(1) @binding(2)
var glyph_sampler: sampler;
@group(1) @binding(3)
var height_texture: texture_2d<f32>;
@group(1) @binding(4)
var height_sampler: sampler;


// Four subpixel taps retain fine ink strokes when the 600px source is minified to table size.
fn sample_glyph(uv: vec2<f32>) -> vec4<f32> {
    let dx = dpdx(uv) * 0.25;
    let dy = dpdy(uv) * 0.25;
    let a = textureSample(glyph_texture, glyph_sampler, clamp(uv - dx - dy, vec2<f32>(0.0), vec2<f32>(1.0)));
    let b = textureSample(glyph_texture, glyph_sampler, clamp(uv + dx - dy, vec2<f32>(0.0), vec2<f32>(1.0)));
    let c = textureSample(glyph_texture, glyph_sampler, clamp(uv - dx + dy, vec2<f32>(0.0), vec2<f32>(1.0)));
    let d = textureSample(glyph_texture, glyph_sampler, clamp(uv + dx + dy, vec2<f32>(0.0), vec2<f32>(1.0)));
    let alpha_sum = a.a + b.a + c.a + d.a;
    let pigment = (a.rgb * a.a + b.rgb * b.a + c.rgb * c.a + d.rgb * d.a) / max(alpha_sum, 0.0001);
    return vec4<f32>(pigment, alpha_sum * 0.25);
}

fn height_at(uv: vec2<f32>) -> f32 {
    return textureSample(height_texture, height_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).r;
}

