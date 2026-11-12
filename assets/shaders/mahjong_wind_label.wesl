import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct MahjongWindLabelMaterial {
    color: vec4<f32>,
    ink_bounds: vec4<f32>,
}

@group(1) @binding(0) var<uniform> material: MahjongWindLabelMaterial;
@group(1) @binding(1) var glyph_texture: texture_2d<f32>;
@group(1) @binding(2) var glyph_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let center = material.ink_bounds.xy + material.ink_bounds.zw * 0.5;
    // All four glyphs have the same visible extent and are centered by their ink.
    let extent = max(material.ink_bounds.z, material.ink_bounds.w) / 0.875;
    let uv = center + (in.uv - 0.5) * extent;
    let dx = dpdx(uv);
    let dy = dpdy(uv);
    var coverage = 0.0;
    // Integrate the coverage mask so glyphs retain smooth edges at 38px.
    for (var y = 0; y < 4; y += 1) {
        for (var x = 0; x < 4; x += 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * 0.25 - 0.375;
            let sample_uv = uv + dx * offset.x + dy * offset.y;
            let ink = textureSample(glyph_texture, glyph_sampler, sample_uv).r;
            coverage += smoothstep(0.1, 0.9, ink);
        }
    }
    return vec4<f32>(material.color.rgb, material.color.a * coverage / 16.0);
}
