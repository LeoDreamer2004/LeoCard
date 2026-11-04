import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct LiquidMaterial {
    tint: vec4<f32>,
    animation: vec4<f32>,
    pointer: vec2<f32>,
    params: vec4<f32>,
    uv_rect: vec4<f32>,
    ripples: array<vec4<f32>, 32>,
}

@group(1) @binding(0) var<uniform> material: LiquidMaterial;
@group(1) @binding(1) var surface_texture: texture_2d<f32>;
@group(1) @binding(2) var surface_sampler: sampler;

fn glyph_region(uv: vec2<f32>, bounds: vec2<f32>) -> f32 {
    let horizontal = smoothstep(bounds.x, bounds.x + 0.025, uv.x)
        * (1.0 - smoothstep(bounds.y - 0.025, bounds.y, uv.x));
    let vertical = smoothstep(0.18, 0.25, uv.y)
        * (1.0 - smoothstep(0.75, 0.83, uv.y));
    return horizontal * vertical;
}

fn join_feedback_uv(uv: vec2<f32>, hover: f32) -> vec2<f32> {
    // Reposition the glyph regions in the original texture; the outer bevel stays fixed.
    let icon_center = vec2<f32>(0.232, 0.505);
    let angle = hover * 0.075;
    let relative = (uv - icon_center - vec2<f32>(-0.008, -0.025) * hover)
        / (1.0 + 0.055 * hover);
    let rotated = vec2<f32>(
        cos(angle) * relative.x - sin(angle) * relative.y / 3.0,
        sin(angle) * relative.x * 3.0 + cos(angle) * relative.y,
    );
    let icon_uv = icon_center + rotated;
    let text_hover = smoothstep(0.10, 1.0, hover);
    let text_center = vec2<f32>(0.615, 0.505);
    let text_uv = text_center
        + (uv - text_center - vec2<f32>(0.008, -0.023) * text_hover)
            / (1.0 + 0.025 * text_hover);
    let icon = mix(uv, icon_uv, glyph_region(uv, vec2<f32>(0.085, 0.355)));
    return mix(icon, text_uv, glyph_region(uv, vec2<f32>(0.335, 0.92)));
}

// Sum signed waves before lighting so peaks reinforce and troughs cancel them.
fn ripple_field(uv: vec2<f32>, metric: vec2<f32>) -> vec3<f32> {
    var height = 0.0;
    var refraction = vec2<f32>(0.0);
    for (var index = 0u; index < u32(material.animation.z); index += 1u) {
        let ripple = material.ripples[index];
        let delta = (uv - ripple.xy) * metric;
        let distance = length(delta);
        let age = ripple.z;
        let travel = distance - age * 0.95;
        let width = travel / (0.12 + age * 0.065);
        let fade = exp(-age * 1.15) * (1.0 - smoothstep(1.5, 1.95, age));
        let wave = cos(travel * 34.0) * exp(-width * width) * fade / sqrt(1.0 + distance * 3.0);
        height += wave;
        refraction += delta / metric / max(distance, 0.04) * wave;
    }
    return vec3<f32>(refraction, height);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    if material.params.x > 0.5 {
        // The join button's icon leads the lettering; its frame remains stationary.
        var content_uv = uv;
        if material.params.x > 1.5 {
            content_uv = join_feedback_uv(uv, material.animation.y);
        }
        let sample_uv = mix(material.uv_rect.xy, material.uv_rect.zw, content_uv);
        let texel = textureSample(surface_texture, surface_sampler, sample_uv);
        if material.params.x < 1.5 {
            // Neutralize the lavender enamel, preserving the cream lettering and arrow.
            let luminance = dot(texel.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
            let lettering = smoothstep(0.0, 0.08, texel.g - texel.b);
            let color = mix(vec3<f32>(luminance * 0.75) * material.tint.rgb, texel.rgb, lettering);
            return vec4<f32>(color, texel.a * material.tint.a * material.animation.w);
        }
        let progress = clamp(fract(material.params.w / 1.05) / 0.78, 0.0, 1.0);
        let eased = progress * progress * (3.0 - 2.0 * progress);
        let x = eased * 1.3 - 0.15;
        let lens = pow(max(sin(uv.y * 3.1415927), 0.0), 0.55);
        let width = 0.013 + lens * 0.021;
        let beam = exp(-pow((uv.x - x) / width, 2.0));
        let halo = exp(-pow((uv.x - x + 0.045) / 0.085, 2.0)) * 0.18;
        let envelope = smoothstep(0.0, 0.08, progress) * (1.0 - smoothstep(0.88, 1.0, progress));
        let color = texel.rgb * material.tint.rgb
            + vec3<f32>(0.46, 0.40, 0.56) * (beam + halo) * envelope * (0.65 + lens * 0.35) * material.params.z;
        return vec4<f32>(color, texel.a * material.tint.a * material.animation.w);
    }
    let aspect = in.size.x / max(in.size.y, 1.0);
    let metric = vec2<f32>(aspect, 1.0);
    let edge = min(min(uv.x * aspect, (1.0 - uv.x) * aspect), min(uv.y, 1.0 - uv.y));
    // Ambient water is almost still; interaction supplies the visible response.
    let cycle = material.animation.x * 6.2831853 / 30.0 + material.params.y;
    let hover = material.animation.y;
    let to_pointer = (material.pointer - uv) * metric;
    let focus = exp(-dot(to_pointer, to_pointer) * 3.8) * hover;
    let interior = smoothstep(0.025, 0.14, edge);
    let wave = sin(uv.x * 10.0 + uv.y * 4.0 - cycle)
        + 0.55 * sin(uv.x * 5.0 - uv.y * 7.0 + cycle * 2.0);
    let flow = vec2<f32>(sin(uv.y * 9.0 + cycle), cos(uv.x * 7.0 - cycle)) * 0.0008;
    let ripples = clamp(ripple_field(uv, metric), vec3<f32>(-1.5), vec3<f32>(1.5));
    let ripple = ripples.z;
    let refraction = flow + to_pointer / metric * focus * 0.035 + ripples.xy * 0.016;
    // Keep the outer bevel pinned; preserve the motif's aspect ratio inside it.
    let dimensions = vec2<f32>(textureDimensions(surface_texture));
    let source_aspect = dimensions.x / dimensions.y;
    let proportional_uv = vec2<f32>((uv.x - 0.5) * aspect / source_aspect + 0.5, uv.y);
    let base_uv = mix(uv, proportional_uv, interior);
    let sample_uv = clamp(base_uv + refraction * interior, vec2<f32>(0.0), vec2<f32>(1.0));
    let texel = textureSample(surface_texture, surface_sampler, sample_uv);
    let silhouette = textureSample(surface_texture, surface_sampler, uv).a;
    var color = texel.rgb * material.tint.rgb;
    let caustic = pow(0.5 + 0.5 * sin(wave * 2.4 + cycle), 7.0);
    let ambient = wave * 0.0008 + caustic * 0.002;
    let light = ambient + focus * 0.17 + max(ripple, 0.0) * 0.11 + min(ripple, 0.0) * 0.018;
    color += (material.tint.rgb * 0.65 + vec3<f32>(0.22)) * light * interior;
    color *= 1.0 - material.params.z * 0.06 * interior;
    return vec4<f32>(max(color, vec3<f32>(0.0)), min(texel.a, silhouette) * material.tint.a * material.animation.w);
}
