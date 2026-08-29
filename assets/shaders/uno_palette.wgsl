#import bevy_ui::ui_vertex_output::UiVertexOutput

struct UnoPaletteMaterial {
    // x: selected sector (red/yellow/green/blue = 0/1/2/3)
    // y: 0 draws the base with that sector removed; 1 draws only that sector
    // z: opacity
    params: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: UnoPaletteMaterial;

const PI: f32 = 3.141592653589793;
const HALF_PI: f32 = 1.5707963267948966;
const TAU: f32 = 6.283185307179586;

fn sector_color(index: f32, dark: bool) -> vec3<f32> {
    // The render target expects linear RGB. These are the linearized forms of
    // the saturated UNO palette used by the rest of the UI.
    if dark {
        if index < 0.5 {
            return vec3<f32>(0.815, 0.047, 0.296);
        }
        if index < 1.5 {
            return vec3<f32>(0.005, 0.342, 0.342);
        }
        if index < 2.5 {
            return vec3<f32>(0.914, 0.157, 0.010);
        }
        return vec3<f32>(0.198, 0.051, 0.539);
    }
    if index < 0.5 {
        return vec3<f32>(0.815, 0.030, 0.024);
    }
    if index < 1.5 {
        return vec3<f32>(0.940, 0.515, 0.010);
    }
    if index < 2.5 {
        return vec3<f32>(0.019, 0.429, 0.067);
    }
    return vec3<f32>(0.014, 0.159, 0.760);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let point = (in.uv - vec2<f32>(0.5)) * 2.0;
    let radius = length(point);
    var angle = atan2(point.y, point.x);
    if angle < 0.0 {
        angle += TAU;
    }
    let sector = min(floor(angle / HALF_PI), 3.0);
    let local_angle = angle - sector * HALF_PI;

    let inner_radius = 0.225;
    let outer_radius = 0.817;
    let gap = 0.055;
    let radial_edge = min(radius - inner_radius, outer_radius - radius);
    let angular_edge = min(local_angle - gap, HALF_PI - gap - local_angle) * radius;
    let edge = min(radial_edge, angular_edge);
    let antialias = max(fwidth(edge) * 1.15, 0.0015);
    let sector_shape = smoothstep(-antialias, antialias, edge);
    let selected_sector = material.params.x % 4.0;
    let is_selected = 1.0 - step(0.5, abs(sector - selected_sector));
    var sector_visibility = 1.0 - is_selected;
    if material.params.y > 0.5 {
        sector_visibility = is_selected;
    }

    let radial = clamp((radius - inner_radius) / (outer_radius - inner_radius), 0.0, 1.0);
    let direction = normalize(point + vec2<f32>(0.0001));
    let directional_light = dot(direction, normalize(vec2<f32>(-0.55, -0.84))) * 0.07;
    let dark_rim = 1.0 - smoothstep(0.0, 0.025, edge);
    let bevel_glint = smoothstep(0.012, 0.032, edge) * (1.0 - smoothstep(0.032, 0.065, edge));
    var color = sector_color(sector, material.params.x >= 4.0) * (0.96 + directional_light - radial * 0.10);
    color = mix(color, color * 0.55, dark_rim);
    color += vec3<f32>(0.018) * bevel_glint;
    let sector_alpha = sector_shape * sector_visibility * material.params.z;

    if material.params.y < 0.5 {
        let hub_edge = 0.200 - radius;
        let hub_antialias = max(fwidth(hub_edge) * 1.15, 0.0015);
        let hub_alpha = smoothstep(-hub_antialias, hub_antialias, hub_edge) * material.params.z;
        if hub_alpha > sector_alpha {
            let hub_radial = clamp(radius / 0.200, 0.0, 1.0);
            let hub_rim = smoothstep(0.72, 0.96, hub_radial);
            let hub_light = dot(direction, normalize(vec2<f32>(-0.55, -0.84))) * 0.025;
            var hub_color = vec3<f32>(0.013, 0.011, 0.008) * (1.0 - hub_radial * 0.24 + hub_light);
            hub_color = mix(hub_color, vec3<f32>(0.078, 0.042, 0.007), hub_rim * 0.72);
            return vec4<f32>(hub_color, hub_alpha);
        }
    }

    return vec4<f32>(color, sector_alpha);
}
