import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct MahjongWinArtMaterial {
    animation: vec4<f32>,
    bounds: vec4<f32>,
    pose: vec4<f32>,
    surface: vec4<f32>,
}

@group(1) @binding(0) var<uniform> material: MahjongWinArtMaterial;
@group(1) @binding(1) var art_texture: texture_2d<f32>;
@group(1) @binding(2) var art_sampler: sampler;

fn hash(point: vec2<f32>) -> f32 {
    return fract(sin(dot(point, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn paper_noise(point: vec2<f32>) -> f32 {
    let cell = floor(point);
    let fraction = fract(point);
    let blend = fraction * fraction * (3.0 - 2.0 * fraction);
    return mix(mix(hash(cell), hash(cell + vec2<f32>(1.0, 0.0)), blend.x),
        mix(hash(cell + vec2<f32>(0.0, 1.0)), hash(cell + vec2<f32>(1.0)), blend.x), blend.y);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let ratio = (in.size.x / max(in.size.y, 1.0)) / material.surface.x;
    if material.animation.w < 0.5 {
        // Cover the viewport while preserving the artwork's original aspect ratio.
        let extent = vec2<f32>(min(ratio, 1.0), min(1.0 / ratio, 1.0));
        let uv = (in.uv - 0.5) * extent / material.animation.z + 0.5;
        let art = textureSample(art_texture, art_sampler, uv);
        let grain = paper_noise(uv * vec2<f32>(47.0, 19.0)) * 0.12
            + paper_noise(uv * vec2<f32>(129.0, 61.0)) * 0.04;
        let sweep = uv.x * 0.72 + uv.y * 0.28 + grain;
        let covered = smoothstep(sweep - 0.06, sweep + 0.06, material.animation.y * 1.4 - 0.12);
        return vec4<f32>(art.rgb, art.a * covered * material.animation.x);
    }
    // Fit the whole calligraphic title, so vertical and two-row titles keep their proportions.
    let extent = vec2<f32>(max(ratio, 1.0), max(1.0 / ratio, 1.0));
    let title_uv = (in.uv - 0.5) * extent + 0.5;
    let center = material.bounds.xy + material.bounds.zw * 0.5;
    let uv = center + (title_uv - center) / material.pose.z - material.pose.xy * material.bounds.zw;
    let ink = textureSample(art_texture, art_sampler, uv);
    let texel = 1.0 / vec2<f32>(textureDimensions(art_texture));
    let nearby = textureSample(art_texture, art_sampler, uv + texel * vec2<f32>(2.0, -1.0));
    let inside = step(material.bounds.x, uv.x) * step(uv.x, material.bounds.x + material.bounds.z)
        * step(material.bounds.y, uv.y) * step(uv.y, material.bounds.y + material.bounds.w);
    let local = (uv - material.bounds.xy) / material.bounds.zw;
    let grain = paper_noise(local * vec2<f32>(31.0, 43.0)) * 0.10;
    let stroke = local.y * 0.82 + local.x * 0.18 + grain;
    let painted = smoothstep(stroke - 0.04, stroke + 0.04, material.animation.y * 1.25 - 0.08);

    // A broad soft band and a narrow ridge travel together across the complete title.
    // Alpha always comes from the pigment, so the moving light cannot expose its rectangular slot.
    let distance = abs(title_uv.x + title_uv.y * 0.18 - material.pose.w);
    let soft = 1.0 - smoothstep(0.025, 0.18, distance);
    let ridge = 1.0 - smoothstep(0.008, 0.045, distance);
    let luminance = dot(ink.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let texture_grain = clamp(length(ink.rgb - nearby.rgb) * 4.0, 0.0, 1.0);
    let polishing = (soft * 0.23 + ridge * 0.17) * (0.72 + texture_grain * 0.28)
        * material.surface.y * (1.0 - luminance * 0.4);
    let color = mix(ink.rgb, vec3<f32>(1.0, 0.94, 0.78), polishing);
    return vec4<f32>(color, ink.a * inside * painted * material.animation.x);
}
