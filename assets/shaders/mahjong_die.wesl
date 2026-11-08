import bevy_ui_render::ui_vertex_output::UiVertexOutput;
import super::mahjong_tile_geometry::box_intersection;

struct MahjongDieMaterial {
    ray_origin: vec4<f32>,
    ray_center: vec4<f32>,
    ray_right: vec4<f32>,
    ray_down: vec4<f32>,
    light: vec4<f32>,
    shadow: vec4<f32>,
    red: u32,
}

@group(1) @binding(0) var<uniform> material: MahjongDieMaterial;

const PIPS = array<u32, 7>(0u, 16u, 257u, 273u, 325u, 341u, 365u);

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let origin = material.ray_origin.xyz;
    let ray = material.ray_center.xyz + material.ray_right.xyz * (in.uv.x - 0.5)
        + material.ray_down.xyz * (in.uv.y - 0.5);
    let hit = box_intersection(origin, ray, vec3<f32>(1.0), 0.16);
    let gap = hit.near - hit.far;
    let aa = clamp(fwidth(gap) * 0.55, 0.001, 0.12);
    let coverage = (1.0 - smoothstep(-aa, aa, gap)) * step(0.0, hit.near);
    let point = origin + ray * hit.near;

    var face = 0u;
    var face_uv = point.xz;
    if abs(hit.normal.y) > 0.99 {
        face = select(6u, 1u, hit.normal.y > 0.0);
    } else if abs(hit.normal.z) > 0.99 {
        face = select(2u, 5u, hit.normal.z > 0.0);
        face_uv = point.xy;
    } else if abs(hit.normal.x) > 0.99 {
        face = select(4u, 3u, hit.normal.x > 0.0);
        face_uv = point.zy;
    }
    let pip_cell = clamp(round(face_uv / 0.47), vec2<f32>(-1.0), vec2<f32>(1.0));
    let pip_index = u32(pip_cell.x + 1.0) + 3u * u32(pip_cell.y + 1.0);
    let pip_radius = select(0.135, 0.20, face == 1u);
    let distance = length(face_uv - pip_cell * 0.47) - pip_radius;
    let pip_aa = clamp(fwidth(distance) * 0.6, 0.001, 0.10);
    let pip = (1.0 - smoothstep(-pip_aa, pip_aa, distance))
        * f32((PIPS[face] >> pip_index) & 1u);

    let normal = normalize(max(abs(point) - 0.84, vec3<f32>(0.0)) * sign(point)
        + hit.normal * 0.0001);
    let diffuse = 0.55 + max(dot(normal, material.light.xyz), 0.0) * 0.45;
    let ivory = vec3<f32>(0.94, 0.89, 0.77);
    let ink = select(vec3<f32>(0.018, 0.022, 0.027), vec3<f32>(0.52, 0.016, 0.023),
        face == 1u || face == 4u);
    let red = material.red != 0u;
    let shell_color = select(ivory, vec3<f32>(0.72, 0.045, 0.06), red);
    let pip_color = select(ink, ivory, red);
    let body = mix(shell_color, pip_color, pip) * diffuse;

    let shadow_distance = length((in.uv - material.shadow.xy) / material.shadow.zw);
    let shadow = (1.0 - smoothstep(0.35, 1.0, shadow_distance)) * 0.40 * (1.0 - coverage);
    let alpha = coverage + shadow;
    return vec4<f32>((body * coverage + vec3<f32>(0.015) * shadow) / max(alpha, 0.0001), alpha);
}
