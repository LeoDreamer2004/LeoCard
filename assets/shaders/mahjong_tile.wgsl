#import bevy_ui::ui_vertex_output::UiVertexOutput

struct MahjongTileMaterial {
    // x: interaction; y: face/back; z: opacity.
    // w: -4 own meld, -3 opposite stand, -2 side stand, -1 own hand, 2 double wall, 3 lower wall tile.
    params: vec4<f32>,
    // xy: screen-space upper-left light transformed into the tile's local orientation.
    lighting: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: MahjongTileMaterial;
@group(1) @binding(1)
var glyph_texture: texture_2d<f32>;
@group(1) @binding(2)
var glyph_sampler: sampler;
@group(1) @binding(3)
var height_texture: texture_2d<f32>;
@group(1) @binding(4)
var height_sampler: sampler;

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

// 将圆角牌面沿厚度方向连续扫掠，得到没有尖角的完整牌体轮廓。
fn swept_rounded_box_distance(
    point: vec2<f32>,
    center: vec2<f32>,
    half_size: vec2<f32>,
    radius: f32,
    extrusion: vec2<f32>,
) -> f32 {
    var distance = rounded_box(point - center, half_size, radius);
    for (var step: i32 = 1; step <= 8; step += 1) {
        let progress = f32(step) / 8.0;
        distance = min(
            distance,
            rounded_box(point - center - extrusion * progress, half_size, radius),
        );
    }
    return distance;
}

fn height_at(uv: vec2<f32>) -> f32 {
    return textureSample(height_texture, height_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).r;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let point = in.uv - vec2<f32>(0.5);
    let wall_mode = step(0.5, material.params.w);
    let double_wall = 1.0 - step(0.45, abs(material.params.w - 2.0));
    let lower_wall = 1.0 - step(0.45, abs(material.params.w - 3.0));
    let own_hand_mode = 1.0 - step(0.45, abs(material.params.w + 1.0));
    let side_stand_mode = 1.0 - step(0.45, abs(material.params.w + 2.0));
    let opposite_stand_mode = 1.0 - step(0.45, abs(material.params.w + 3.0));
    let own_meld_mode = 1.0 - step(0.45, abs(material.params.w + 4.0));
    var face_center = vec2<f32>(-0.025, -0.045);
    face_center = mix(face_center, vec2<f32>(-0.025, 0.045), own_hand_mode);
    face_center = mix(face_center, vec2<f32>(-0.010, -0.025), side_stand_mode);
    face_center = mix(face_center, vec2<f32>(-0.025, -0.055), opposite_stand_mode);
    face_center = mix(face_center, vec2<f32>(-0.025, -0.090), own_meld_mode);
    face_center = mix(face_center, vec2<f32>(-0.040, -0.110), wall_mode);
    face_center += vec2<f32>(0.030, 0.140) * lower_wall;
    var face_half_size = vec2<f32>(0.430, 0.405);
    face_half_size = mix(face_half_size, vec2<f32>(0.435, 0.185), side_stand_mode);
    face_half_size = mix(face_half_size, vec2<f32>(0.430, 0.355), opposite_stand_mode);
    face_half_size = mix(face_half_size, vec2<f32>(0.430, 0.330), own_meld_mode);
    face_half_size = mix(face_half_size, vec2<f32>(0.395, 0.280), wall_mode);
    let regular_extrusion = vec2<f32>(0.055, 0.090);
    let own_hand_extrusion = vec2<f32>(0.055, -0.090);
    let side_stand_extrusion = vec2<f32>(0.020, 0.105);
    let opposite_stand_extrusion = vec2<f32>(0.045, 0.105);
    let own_meld_extrusion = vec2<f32>(0.055, 0.140);
    let wall_extrusion = mix(vec2<f32>(0.055, 0.150), vec2<f32>(0.085, 0.280), double_wall);
    var extrusion = mix(regular_extrusion, own_hand_extrusion, own_hand_mode);
    extrusion = mix(extrusion, side_stand_extrusion, side_stand_mode);
    extrusion = mix(extrusion, opposite_stand_extrusion, opposite_stand_mode);
    extrusion = mix(extrusion, own_meld_extrusion, own_meld_mode);
    extrusion = mix(extrusion, wall_extrusion, wall_mode);
    var face_radius = mix(0.075, 0.085, own_hand_mode);
    face_radius = mix(face_radius, 0.090, side_stand_mode);
    face_radius = mix(face_radius, 0.085, opposite_stand_mode);
    face_radius = mix(face_radius, 0.085, own_meld_mode);
    face_radius = mix(face_radius, 0.070, wall_mode);
    let face_point = point - face_center;
    let face_distance = rounded_box(face_point, face_half_size, face_radius);
    let face = mask_from_distance(face_distance);
    let body_distance = swept_rounded_box_distance(
        point,
        face_center,
        face_half_size,
        face_radius,
        extrusion,
    );
    let side = mask_from_distance(body_distance);
    let side_only = side * (1.0 - face);
    let alpha = max(face, side_only);

    // 牌背的绿色只属于顶面嵌层，牌体及其侧壁均为白瓷。
    // 前侧面更暗，右侧面承接左上方光源，棱边带有釉面高光与接触暗线。
    let local_light = normalize(material.lighting.xy);
    let side_light = clamp(0.72 + dot(point, local_light) * 0.28, 0.0, 1.0);
    let porcelain_front = mix(
        vec3<f32>(0.47, 0.44, 0.36),
        vec3<f32>(0.79, 0.76, 0.65),
        side_light,
    );
    let porcelain_right = mix(
        vec3<f32>(0.53, 0.51, 0.45),
        vec3<f32>(0.85, 0.83, 0.75),
        side_light,
    );
    let corner_vector = max(
        abs(face_point) - face_half_size + vec2<f32>(face_radius),
        vec2<f32>(0.0),
    ) * sign(face_point);
    let oriented_outward = max(corner_vector * sign(extrusion), vec2<f32>(0.0));
    let lateral_weight = oriented_outward.x
        / max(oriented_outward.x + oriented_outward.y, 0.0001);
    let depth_weight = 1.0 - lateral_weight;
    let porcelain_top = mix(
        vec3<f32>(0.62, 0.60, 0.54),
        vec3<f32>(0.88, 0.86, 0.78),
        side_light,
    );
    let upward_surface = max(
        max(max(own_hand_mode, side_stand_mode), opposite_stand_mode),
        own_meld_mode,
    );
    let depth_color = mix(porcelain_front, porcelain_top, upward_surface);
    var color = mix(depth_color, porcelain_right, lateral_weight);
    let porcelain_grain = sin(point.x * 181.0 + sin(point.y * 137.0) * 2.3) * 0.007;
    color *= 1.0 + porcelain_grain;
    let outer_bevel = 1.0 - smoothstep(0.0, 0.060, max(-body_distance, 0.0));
    let top_seam = 1.0 - smoothstep(0.0, 0.040, max(face_distance, 0.0));
    let side_specular = outer_bevel
        * clamp(0.78 + dot(point, local_light) * 0.68, 0.0, 1.0)
        * 0.27;
    color += vec3<f32>(side_specular + top_seam * 0.015);
    let side_glaze = pow(
        clamp(1.0 - abs(point.x * 0.58 + point.y * 0.42 - 0.08) * 3.4, 0.0, 1.0),
        5.0,
    ) * side_only;
    color += vec3<f32>(side_glaze * 0.095);
    color *= 1.0 - depth_weight * mix(0.10, 0.025, upward_surface);
    color += vec3<f32>(0.0, 0.018, 0.010) * depth_weight * (1.0 - upward_surface);

    // 双层牌墙只绘制一次顶面；侧壁中央的暗线表示两张白瓷牌的接缝。
    let bottom = face_center.y + face_half_size.y;
    let right = face_center.x + face_half_size.x;
    let front_layer_seam = side_only * depth_weight
        * (1.0 - smoothstep(0.006, 0.020,
            abs(point.y - (bottom + extrusion.y * 0.5))));
    let right_layer_seam = side_only * lateral_weight
        * (1.0 - smoothstep(0.006, 0.020,
            abs(point.x - (right + extrusion.x * 0.5))));
    let layer_seam = max(front_layer_seam, right_layer_seam) * double_wall;
    color *= 1.0 - layer_seam * 0.24;
    color += vec3<f32>(layer_seam * 0.025);

    let face_uv = face_point / (face_half_size * 2.0) + vec2<f32>(0.5);

    let dimensions = max(vec2<f32>(textureDimensions(height_texture)), vec2<f32>(1.0));
    let texel = vec2<f32>(2.35) / dimensions;
    let height = height_at(face_uv);
    let gradient = vec2<f32>(
        height_at(face_uv + vec2<f32>(texel.x, 0.0)) - height_at(face_uv - vec2<f32>(texel.x, 0.0)),
        height_at(face_uv - vec2<f32>(0.0, texel.y)) - height_at(face_uv + vec2<f32>(0.0, texel.y)),
    );
    let groove_edge = clamp(length(gradient) * 3.3, 0.0, 1.0);
    let normal = normalize(vec3<f32>(gradient.x * 3.2, gradient.y * 3.2, 1.0));
    let light = normalize(vec3<f32>(material.lighting.xy, 1.30));
    let half_vector = normalize(light + vec3<f32>(0.0, 0.0, 1.0));
    let diffuse = 0.79 + 0.21 * max(dot(normal, light), 0.0);
    let specular = pow(max(dot(normal, half_vector), 0.0), 38.0) * (0.13 + groove_edge * 0.34);

    let edge_depth = clamp(-face_distance / 0.075, 0.0, 1.0);
    let bevel = smoothstep(0.0, 0.82, edge_depth);
    let bevel_zone = 1.0 - bevel;
    let bevel_direction = normalize(vec3<f32>(
        -corner_vector.x,
        -corner_vector.y,
        0.72,
    ));
    let bevel_light = max(dot(bevel_direction, light), 0.0);
    let warmth = clamp(0.58 - point.y * 0.22 - point.x * 0.05, 0.0, 1.0);
    let ivory = mix(vec3<f32>(0.72, 0.68, 0.57), vec3<f32>(0.94, 0.91, 0.79), warmth);
    let surface_variation = sin(face_uv.x * 173.0 + sin(face_uv.y * 127.0) * 2.7) * 0.006;
    var face_color = ivory * (1.0 + surface_variation) * diffuse
        * (0.965 + bevel * 0.035)
        * (1.0 - bevel_zone * 0.035 + bevel_zone * bevel_light * 0.055)
        + vec3<f32>(specular + bevel_zone * bevel_light * 0.025);
    let glaze_band = pow(
        clamp(1.0 - abs(face_uv.x * 0.62 + face_uv.y * 0.38 - 0.32) * 3.1, 0.0, 1.0),
        6.0,
    ) * smoothstep(0.02, 0.16, face_uv.x)
        * smoothstep(0.02, 0.16, face_uv.y)
        * smoothstep(0.02, 0.16, 1.0 - face_uv.x)
        * smoothstep(0.02, 0.16, 1.0 - face_uv.y);
    face_color += vec3<f32>(glaze_band * 0.055);

    let glyph = textureSample(glyph_texture, glyph_sampler, clamp(face_uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    let engraving_shadow = height * 0.11 + groove_edge * 0.035;
    face_color *= 1.0 - engraving_shadow;
    let pigment_variation = 0.96
        + sin(face_uv.x * 211.0 + face_uv.y * 157.0) * 0.018
        - height * 0.055;
    let lacquer_edge = groove_edge
        * clamp(dot(normal.xy, local_light) + 0.35, 0.0, 1.0)
        * 0.12;
    let ink = glyph.rgb * (0.86 + diffuse * 0.12) * pigment_variation
        + vec3<f32>(lacquer_edge);
    face_color = mix(face_color, ink, glyph.a);
    face_color += vec3<f32>(specular * (1.0 - glyph.a * 0.70));

    let back_mode = step(0.5, material.params.y);
    let back_inset_half_size = face_half_size - vec2<f32>(0.048);
    let back_inset_distance = rounded_box(face_point, back_inset_half_size, 0.040);
    let back_inset = mask_from_distance(back_inset_distance);
    let back_rim = clamp(1.0 - abs(back_inset_distance) / 0.030, 0.0, 1.0);
    let back_pattern = glyph.a;
    let back_base = mix(
        vec3<f32>(0.018, 0.190, 0.140),
        vec3<f32>(0.035, 0.315, 0.235),
        0.42 + back_pattern * 0.45,
    );
    let back_light = 0.84 + 0.16 * clamp(0.70 + dot(point, local_light) * 0.52, 0.0, 1.0);
    var back_color = mix(ivory * 0.78, back_base * back_light, back_inset);
    let back_glaze = glaze_band * back_inset;
    back_color += vec3<f32>(back_rim * 0.075 + specular * 0.55 + back_glaze * 0.105);
    back_color += vec3<f32>(0.0, back_glaze * 0.018, back_glaze * 0.012);
    face_color = mix(face_color, back_color, back_mode);

    let interaction_amount = step(0.5, material.params.x);
    let pressed = step(1.5, material.params.x);
    let interactive = mix(face_color, face_color * vec3<f32>(0.90, 0.98, 0.92), 0.18 + pressed * 0.18)
        + vec3<f32>(0.035, 0.055, 0.025) * (1.0 - pressed);
    face_color = mix(face_color, interactive, interaction_amount);
    color = mix(color, face_color, face);

    return vec4<f32>(color, alpha * material.params.z);
}
