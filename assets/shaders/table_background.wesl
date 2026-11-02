import bevy_ui_render::ui_vertex_output::UiVertexOutput;

const CARD_TABLE_CLOSEUP: f32 = 1.45;

struct TableBackgroundMaterial {
    // x: brightness, y: 0 centered oval card table, 1 measured rectangular mahjong table.
    params: vec4<f32>,
    felt_bounds: vec4<f32>,
    frame_bounds: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: TableBackgroundMaterial;
@group(1) @binding(1)
var table_texture: texture_2d<f32>;
@group(1) @binding(2)
var table_sampler: sampler;
@group(1) @binding(3)
var rail_texture: texture_2d<f32>;
@group(1) @binding(4)
var rail_sampler: sampler;

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

fn table_uv(pixel: vec2<f32>, size: vec2<f32>, rail_size: vec2<f32>) -> vec2<f32> {
    if material.params.y < 0.5 {
        // Center-crop an oversized ellipse; the wooden rim keeps one scale on both axes.
        let fit = max(size.x / rail_size.x, size.y / rail_size.y) * CARD_TABLE_CLOSEUP;
        let extent = rail_size * fit;
        return (pixel - (size - extent) * 0.5) / extent;
    }
    // Match the tile camera; replacing the fabric never changes this rectangular table plane.
    let fit = min(size.x / 2204.0, size.y / 1217.0);
    let reference = (pixel - vec2<f32>((size.x - 2204.0 * fit) * 0.5, 0.0)) / fit;
    let image = (reference - vec2<f32>(1095.0, 608.5)) / 2491.0183;
    let sine = 0.724257;
    let cosine = 0.689530;
    let ray = vec3<f32>(image.x, -sine - image.y * cosine, cosine - image.y * sine);
    let ground = vec3<f32>(0.0, 27.708225, -29.475252) + ray * (-27.708225 / ray.y);
    return vec2<f32>(0.5 + ground.x / 32.0, 0.5 - ground.z / 32.0);
}

fn shape_coverage(uv: vec2<f32>, bounds: vec4<f32>, rounded: bool) -> f32 {
    let half_extent = (bounds.zw - bounds.xy) * 0.5;
    let center = (bounds.xy + bounds.zw) * 0.5;
    var distance: f32;
    if rounded {
        let radius = 0.035;
        let point = abs(uv - center) - half_extent + vec2<f32>(radius);
        distance = length(max(point, vec2<f32>(0.0))) + min(max(point.x, point.y), 0.0) - radius;
    } else {
        distance = length((uv - center) / half_extent) - 1.0;
    }
    let feather = max(fwidth(distance), 0.00001);
    return 1.0 - smoothstep(-feather, feather, distance);
}

fn illuminate_fabric(color: vec3<f32>, uv: vec2<f32>, aspect: f32) -> vec3<f32> {
    var lamp = vec3<f32>(0.0, 0.0, 0.85);
    if material.params.y > 0.5 {
        lamp = vec3<f32>(-0.7, -0.7, 0.9);
    }
    let point = vec3<f32>((uv - 0.5) * vec2<f32>(aspect, 1.0), 0.0);
    let light = normalize(lamp - point);
    let halfway = normalize(light + vec3<f32>(0.0, 0.0, 1.0));
    let diffuse = max(light.z, 0.0);
    // A restrained cloth sheen is additive lighting, with no screen-edge darkening mask.
    let sheen = pow(max(halfway.z, 0.0), 24.0) * 0.055;
    return color * (0.92 + 0.08 * diffuse) + vec3<f32>(1.0, 0.96, 0.87) * sheen;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let dimensions = vec2<f32>(textureDimensions(table_texture));
    let rail_size = vec2<f32>(textureDimensions(rail_texture));
    let uv = table_uv(in.uv * in.size, in.size, rail_size);
    let rounded = material.params.y > 0.5;
    let felt_extent = material.felt_bounds.zw - material.felt_bounds.xy;
    let felt_uv = (uv - material.felt_bounds.xy) / felt_extent;
    let fabric_size = rail_size * felt_extent;
    let sample_uv = cover_uv(felt_uv, fabric_size, dimensions);
    let fabric = textureSample(table_texture, table_sampler, clamp(sample_uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    let lit = illuminate_fabric(fabric.rgb, felt_uv, fabric_size.x / fabric_size.y);
    // Tuck the fabric under the opaque inner trim to prevent a one-pixel seam.
    let padded_bounds = material.felt_bounds + vec4<f32>(-0.008, -0.008, 0.008, 0.008);
    let cloth = shape_coverage(uv, padded_bounds, rounded);
    let outside = vec3<f32>(0.012, 0.018, 0.020);
    var color = mix(outside, lit, cloth);
    let rail = textureSample(rail_texture, rail_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    let silhouette = shape_coverage(uv, material.frame_bounds, rounded);
    let alpha = smoothstep(0.12, 0.95, rail.a) * silhouette;
    color = mix(color, rail.rgb, alpha);
    return vec4<f32>(color * material.params.x, 1.0);
}
