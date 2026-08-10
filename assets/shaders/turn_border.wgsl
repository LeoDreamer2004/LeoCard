#import bevy_ui::ui_vertex_output::UiVertexOutput

struct TurnBorderMaterial {
    // x: normalized tail, y: normalized head, z: corner radius, w: thickness.
    params: vec4<f32>,
    color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: TurnBorderMaterial;

const PI: f32 = 3.141592653589793;

fn rounded_rect_distance(point: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(point) - (half_size - vec2<f32>(radius));
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

// Returns clockwise distance along a rounded rectangle, starting at top center.
fn rounded_rect_path_distance(point: vec2<f32>, half_size: vec2<f32>, radius: f32) -> vec2<f32> {
    let straight_half = half_size.x - radius;
    let vertical = 2.0 * (half_size.y - radius);
    let quarter = 0.5 * PI * radius;
    let total = 4.0 * straight_half + 2.0 * vertical + 4.0 * quarter;
    var distance = 0.0;

    if point.x > straight_half && point.y < -half_size.y + radius {
        let center = vec2<f32>(straight_half, -half_size.y + radius);
        let angle = atan2(point.y - center.y, point.x - center.x);
        distance = straight_half + (angle + 0.5 * PI) * radius;
    } else if point.x > straight_half && point.y > half_size.y - radius {
        let center = vec2<f32>(straight_half, half_size.y - radius);
        let angle = atan2(point.y - center.y, point.x - center.x);
        distance = straight_half + quarter + vertical + angle * radius;
    } else if point.x < -straight_half && point.y > half_size.y - radius {
        let center = vec2<f32>(-straight_half, half_size.y - radius);
        let angle = atan2(point.y - center.y, point.x - center.x);
        distance = 3.0 * straight_half + vertical + 2.0 * quarter
            + (angle - 0.5 * PI) * radius;
    } else if point.x < -straight_half && point.y < -half_size.y + radius {
        let center = vec2<f32>(-straight_half, -half_size.y + radius);
        var angle = atan2(point.y - center.y, point.x - center.x);
        if angle < 0.0 {
            angle += 2.0 * PI;
        }
        distance = 3.0 * straight_half + 2.0 * vertical + 3.0 * quarter
            + (angle - PI) * radius;
    } else if point.y < 0.0 && abs(point.x) <= straight_half {
        if point.x >= 0.0 {
            distance = point.x;
        } else {
            distance = total + point.x;
        }
    } else if point.x >= 0.0 && abs(point.y) <= half_size.y - radius {
        distance = straight_half + quarter + point.y + half_size.y - radius;
    } else if point.y >= 0.0 && abs(point.x) <= straight_half {
        distance = straight_half + vertical + 2.0 * quarter + straight_half - point.x;
    } else {
        distance = 3.0 * straight_half + vertical + 3.0 * quarter
            + half_size.y - radius - point.y;
    }
    return vec2<f32>(distance, max(total, 1.0));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    if material.params.y - material.params.x <= 0.0001 {
        return vec4<f32>(0.0);
    }
    let inset = material.params.w * 0.5 + 0.75;
    let half_size = max(in.size * 0.5 - vec2<f32>(inset), vec2<f32>(1.0));
    let radius = clamp(material.params.z - inset, 1.0, min(half_size.x, half_size.y));
    let point = (in.uv - vec2<f32>(0.5)) * in.size;
    let distance = rounded_rect_distance(point, half_size, radius);
    let antialias = max(fwidth(distance), 0.75);
    let line = 1.0 - smoothstep(
        material.params.w * 0.5 - antialias,
        material.params.w * 0.5 + antialias,
        abs(distance),
    );

    let path = rounded_rect_path_distance(point, half_size, radius);
    let position = path.x / path.y;
    let path_antialias = max(1.0 / path.y, 0.0015);
    let after_tail = smoothstep(
        material.params.x - path_antialias,
        material.params.x + path_antialias,
        position,
    );
    let before_head = 1.0 - smoothstep(
        material.params.y - path_antialias,
        material.params.y + path_antialias,
        position,
    );
    let visible = line * after_tail * before_head;
    return vec4<f32>(material.color.rgb, material.color.a * visible);
}
