import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct TableCardMaterial {
    // xy: card dimensions, z: far-edge width ratio, w: rounded paper corner radius.
    shape: vec4<f32>,
    tint: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: TableCardMaterial;
@group(1) @binding(1)
var card_texture: texture_2d<f32>;
@group(1) @binding(2)
var card_sampler: sampler;

fn card_distance(p: vec2<f32>) -> f32 {
    let size = material.shape.xy;
    let vertical = clamp(p.y / size.y + 0.5, 0.0, 1.0);
    let taper = mix(material.shape.z, 1.0, vertical);
    let q = abs(vec2<f32>(p.x / taper, p.y)) - size * 0.5 + material.shape.w;
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - material.shape.w;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // Normalise to quad height so logical UI scale and phone pixel density change no geometry.
    let p = (in.uv - vec2<f32>(0.5)) * vec2<f32>(in.size.x / in.size.y, 1.0);
    let size = material.shape.xy;
    let vertical = (p.y / size.y) + 0.5;
    let taper = mix(material.shape.z, 1.0, vertical);
    // Inverse homography: far edge is narrower and artwork compresses towards the far edge.
    let uv = vec2<f32>(p.x / (size.x * taper) + 0.5, vertical / taper);
    let texel = textureSample(card_texture, card_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    let distance = card_distance(p);
    let aa = max(fwidth(distance), 0.0005);
    let coverage = 1.0 - smoothstep(-aa, aa, distance);

    // Short soft shadow and a narrow paper-thickness edge beneath the projected silhouette.
    let shadow_distance = card_distance(p - vec2<f32>(0.012, 0.025));
    let shadow_alpha = 0.36 * (1.0 - smoothstep(-0.01, 0.055, shadow_distance));
    let edge_distance = card_distance(p - vec2<f32>(0.0, 0.012));
    let edge_alpha = 1.0 - smoothstep(-aa, aa, edge_distance);
    let face_alpha = texel.a * coverage;
    let under_alpha = edge_alpha + shadow_alpha * (1.0 - edge_alpha);
    let alpha = face_alpha + under_alpha * (1.0 - face_alpha);
    let edge_color = vec3<f32>(0.38, 0.40, 0.40);
    let premultiplied = texel.rgb * material.tint.rgb * face_alpha
        + edge_color * edge_alpha * (1.0 - face_alpha);
    return vec4<f32>(premultiplied / max(alpha, 0.0001), alpha * material.tint.a);
}
