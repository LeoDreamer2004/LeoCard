fn rounded_box(point: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let offset = abs(point) - half_size + vec2<f32>(radius);
    return length(max(offset, vec2<f32>(0.0)))
        + min(max(offset.x, offset.y), 0.0)
        - radius;
}

fn mask_from_distance(distance: f32) -> f32 {
    let antialias = max(fwidth(distance) * 1.15, 0.0008);
    return 1.0 - smoothstep(-antialias, antialias, distance);
}

// Physical depth uses screen coordinates, independently of the tile or seat rotation.
fn local_from_screen(vector: vec2<f32>, angle: f32) -> vec2<f32> {
    let cosine = cos(angle);
    let sine = sin(angle);
    return vec2<f32>(cosine * vector.x + sine * vector.y, -sine * vector.x + cosine * vector.y);
}

fn swept_rounded_box_distance(
    point: vec2<f32>, center: vec2<f32>, half_size: vec2<f32>,
    radius: f32, extrusion: vec2<f32>,
) -> f32 {
    // A rounded hexagonal hull gives a continuous prism edge without stacking sampled silhouettes.
    let direction = select(vec2<f32>(-1.0), vec2<f32>(1.0), extrusion >= vec2<f32>(0.0));
    let p = (point - center) * direction;
    let e = abs(extrusion);
    let b = half_size - vec2<f32>(radius);
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(-b.x, -b.y), vec2<f32>(b.x, -b.y),
        vec2<f32>(b.x + e.x, -b.y + e.y), vec2<f32>(b.x + e.x, b.y + e.y),
        vec2<f32>(-b.x + e.x, b.y + e.y), vec2<f32>(-b.x, b.y),
    );
    var distance_squared = 100.0;
    var inside = true;
    for (var index = 0u; index < 6u; index += 1u) {
        let start = corners[index];
        let edge = corners[(index + 1u) % 6u] - start;
        let offset = p - start;
        let nearest = offset - edge * clamp(dot(offset, edge) / max(dot(edge, edge), 0.000001), 0.0, 1.0);
        distance_squared = min(distance_squared, dot(nearest, nearest));
        inside = inside && edge.x * offset.y - edge.y * offset.x >= 0.0;
    }
    return sqrt(distance_squared) * select(1.0, -1.0, inside) - radius;
}

