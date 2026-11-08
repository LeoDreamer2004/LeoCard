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


struct BoxInterval { near: f32, far: f32, normal: vec3<f32> }

fn clip_bevel(normal: vec3<f32>, limit: f32, origin: vec3<f32>, ray: vec3<f32>, hit: ptr<function, BoxInterval>) {
    let denominator = dot(ray, normal);
    let safe = select(-1.0, 1.0, denominator >= 0.0) * max(abs(denominator), 0.000001);
    let distance = (limit - dot(origin, normal)) / safe;
    if denominator < 0.0 {
        if distance > (*hit).near { (*hit).near = distance; (*hit).normal = normal; }
    } else { (*hit).far = min((*hit).far, distance); }
}

fn box_intersection(origin: vec3<f32>, ray: vec3<f32>, half: vec3<f32>, bevel: f32) -> BoxInterval {
    let safe_ray = select(vec3<f32>(-1.0), vec3<f32>(1.0), ray >= vec3<f32>(0.0)) * max(abs(ray), vec3<f32>(0.000001));
    let a = (-half - origin) / safe_ray;
    let b = (half - origin) / safe_ray;
    let near = min(a, b); let far = max(a, b);
    var hit: BoxInterval;
    hit.near = max(max(near.x, near.y), near.z);
    hit.far = min(min(far.x, far.y), far.z);
    hit.normal = vec3<f32>(-sign(ray.x), 0.0, 0.0);
    if near.y > near.x && near.y >= near.z { hit.normal = vec3<f32>(0.0, -sign(ray.y), 0.0); }
    if near.z > near.x && near.z > near.y { hit.normal = vec3<f32>(0.0, 0.0, -sign(ray.z)); }
    for (var i = 0u; i < 4u; i += 1u) {
        let x = select(-1.0, 1.0, (i & 1u) != 0u);
        let y = select(-1.0, 1.0, (i & 2u) != 0u);
        clip_bevel(vec3<f32>(x,y,0.0) * 0.70710678, (half.x+half.y-bevel)*0.70710678, origin, ray, &hit);
        clip_bevel(vec3<f32>(x,0.0,y) * 0.70710678, (half.x+half.z-bevel)*0.70710678, origin, ray, &hit);
        clip_bevel(vec3<f32>(0.0,x,y) * 0.70710678, (half.y+half.z-bevel)*0.70710678, origin, ray, &hit);
    }
    return hit;
}
