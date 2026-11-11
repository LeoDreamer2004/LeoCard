import bevy_ui_render::ui_vertex_output::UiVertexOutput;

const TAU: f32 = 6.28318530718;

struct UnoDirectionMaterial {
    canvas: vec4<f32>,
    shape: vec4<f32>,
    motion: vec4<f32>,
    color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: UnoDirectionMaterial;

fn cross2(left: vec2<f32>, right: vec2<f32>) -> f32 {
    return left.x * right.y - left.y * right.x;
}

fn edge_distance_squared(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
    let edge = end - start;
    let offset = point - start;
    let nearest = offset - edge * clamp(dot(offset, edge) / dot(edge, edge), 0.0, 1.0);
    return dot(nearest, nearest);
}

// A rounded triangle in arc-length / ellipse-normal coordinates bends with the ring.
fn arrow_distance(point: vec2<f32>, length: f32, half_width: f32, rounding: f32) -> f32 {
    let tip = vec2<f32>(rounding, 0.0);
    let left = vec2<f32>(length - rounding, half_width - rounding);
    let right = vec2<f32>(length - rounding, -half_width + rounding);
    let distance = sqrt(min(
        edge_distance_squared(point, tip, left),
        min(edge_distance_squared(point, left, right), edge_distance_squared(point, right, tip)),
    ));
    let sides = vec3<f32>(
        cross2(left - tip, point - tip),
        cross2(right - left, point - left),
        cross2(tip - right, point - right),
    );
    let inside = all(sides >= vec3<f32>(0.0)) || all(sides <= vec3<f32>(0.0));
    return select(distance, -distance, inside) - rounding;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let point = in.uv * material.canvas.xy - material.canvas.zw;
    let radii = material.shape.xy;
    let normalized = point / radii;
    let radius = length(normalized);
    let gradient = length(point / (radii * radii));
    let signed_distance = select(
        -min(radii.x, radii.y),
        radius * (radius - 1.0) / max(gradient, 0.000001),
        radius > 0.0001,
    );
    let distance = abs(signed_distance);
    let antialias = max(fwidth(distance), 0.65);
    let half_width = material.shape.z * 0.5;
    let band = 1.0 - smoothstep(half_width - antialias, half_width + antialias, distance);
    let angle = fract(atan2(normalized.y, normalized.x + 0.000001) / TAU + 1.0);
    let behind = fract((material.motion.x - angle) * material.motion.y + 1.0);
    let head_angle = material.motion.x * TAU;
    let tangent_length = length(vec2<f32>(-radii.x * sin(head_angle), radii.y * cos(head_angle)));
    let head_length = material.shape.z * 1.75;
    let leading_gap = head_length / (TAU * tangent_length);
    let body_start = smoothstep(leading_gap * 0.35, leading_gap * 0.65, behind);
    let body_end = 1.0 - smoothstep(0.935, 0.955, behind);
    let trail = mix(0.28, 0.82, pow(1.0 - behind, 0.65));
    let body = band * body_start * body_end * trail;

    let head_distance = arrow_distance(
        vec2<f32>(behind * TAU * tangent_length, signed_distance),
        head_length,
        material.shape.z * 0.85,
        material.shape.z * 0.075,
    );
    // Arc coordinates wrap ahead of the tip; keep that derivative jump from drawing a seam.
    let pixel_width = max(max(fwidth(point.x), fwidth(point.y)), 0.65);
    let head_antialias = clamp(fwidth(head_distance), 0.65, pixel_width * 1.5);
    let head = (1.0 - smoothstep(-head_antialias, head_antialias, head_distance))
        * material.motion.z * 1.55;
    let alpha = max(body, head) * material.color.a * material.motion.w;
    return vec4<f32>(material.color.rgb, alpha);
}
