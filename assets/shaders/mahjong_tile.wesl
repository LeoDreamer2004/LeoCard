import bevy_ui_render::ui_vertex_output::UiVertexOutput;
import super::mahjong_tile_bindings::{material, sample_glyph, height_at, height_texture};
import super::mahjong_tile_geometry::{rounded_box, mask_from_distance, local_from_screen, swept_rounded_box_distance};
import super::mahjong_tile_projected::projected_tile;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    if material.ray_origin.w > 0.0 { return projected_tile(in); }
    // The surface extends beyond the layout slot so shadows are not cut off at its edges.
    var point = (in.uv - vec2<f32>(0.5)) * 1.5;
    let tile_size = in.size / 1.5;
    let short_edge = min(tile_size.x, tile_size.y);
    let wall_mode = step(0.5, material.params.w);
    let double_wall = 1.0 - step(0.45, abs(material.params.w - 2.0));
    let own_hand_mode = 1.0 - step(0.45, abs(material.params.w + 6.0));
    let guide_hand_mode = 1.0-step(0.45,abs(material.params.w+1.0));
    let side_stand_mode = 1.0 - step(0.45, abs(material.params.w + 2.0));
    let opposite_stand_mode = 1.0 - step(0.45, abs(material.params.w + 3.0));
    let own_meld_mode = 1.0 - step(0.45, abs(material.params.w + 4.0));
    let concealed_meld_mode = 1.0 - step(0.45, abs(material.params.w + 5.0));
    let standing = max(max(own_hand_mode,guide_hand_mode), max(side_stand_mode, opposite_stand_mode));
    // The side hands are upright prisms: the green inward face leans away from its white top.
    // Apply the same shear to the whole solid, including its shadow, rather than rotating a flat back.
    let shear = sin(material.lighting.w) * side_stand_mode * 0.20;
    point.x -= point.y * shear;
    var face_half_size = vec2<f32>(0.435, 0.395);
    face_half_size = mix(face_half_size,vec2<f32>(0.435,0.435),guide_hand_mode);
    face_half_size = mix(face_half_size, vec2<f32>(0.5, 0.5), own_hand_mode);
    face_half_size = mix(face_half_size, vec2<f32>(0.435, 0.300), side_stand_mode);
    face_half_size = mix(face_half_size, vec2<f32>(0.435, 0.390), opposite_stand_mode);
    face_half_size = mix(face_half_size, vec2<f32>(0.435, 0.330), own_meld_mode);
    face_half_size = mix(face_half_size, vec2<f32>(0.435, 0.310), concealed_meld_mode);
    face_half_size = mix(face_half_size, vec2<f32>(0.380, 0.260), wall_mode);
    var depth = mix(0.24, 0.19, standing);
    depth = mix(depth, 0.16, own_meld_mode);
    depth = mix(depth, 0.20, concealed_meld_mode);
    depth = mix(depth, mix(0.23, 0.44, double_wall), wall_mode);
    // Upright hands reveal their upper rim; tiles lying on the table reveal their lower edge.
    let depth_direction = mix(1.0, -1.0, standing);
    var screen_depth = vec2<f32>(0.020, depth * depth_direction);
    // Side seats expose a tall white side wall instead of the broad lower edge of a face-down tile.
    let outward = -sin(material.lighting.w);
    screen_depth = mix(screen_depth, vec2<f32>(outward * 0.30, 0.10), side_stand_mode);
    screen_depth = mix(screen_depth,vec2<f32>(0.020,-0.100),guide_hand_mode);
    screen_depth = mix(screen_depth, vec2<f32>(0.0, -0.115), own_hand_mode);
    var extrusion = local_from_screen(short_edge * screen_depth, material.lighting.w) / tile_size;
    extrusion.x -= extrusion.y * shear;
    // Own-hand nodes include both the visible front and its upper rim, so their entire body is pickable.
    face_half_size.y -= abs(extrusion.y)*0.5*own_hand_mode;
    let face_center = -extrusion * 0.5;
    let face_radius = mix(0.080, 0.095, standing);
    let face_point = point - face_center;
    let face_distance = rounded_box(face_point, face_half_size, face_radius);
    let face = mask_from_distance(face_distance);
    let body_distance = swept_rounded_box_distance(point, face_center, face_half_size, face_radius, extrusion);
    let side = mask_from_distance(body_distance);
    let side_only = side * (1.0 - face);
    let tile_alpha = max(face, side_only);

    // A short dense contact shadow and wider soft penumbra follow the silhouette, not the UI rectangle.
    var shadow_offset = local_from_screen(
        short_edge * vec2<f32>(0.055, 0.16) + vec2<f32>(0.15, 1.30) * material.elevation,
        material.lighting.w,
    ) / tile_size;
    shadow_offset.x -= shadow_offset.y * shear;
    var shadow_center = face_center;
    var shadow_half_size = face_half_size;
    var shadow_extrusion = extrusion;
    var shadow_radius = face_radius;
    if standing > 0.5 {
        // An upright face casts from its foot on the cloth, not from its raised white roof.
        shadow_center.y += face_half_size.y * mix(-0.88, 0.88, max(own_hand_mode,guide_hand_mode));
        shadow_half_size.y = 0.065;
        shadow_extrusion = vec2<f32>(0.0);
        shadow_radius = 0.045;
    }
    let shadow_distance = swept_rounded_box_distance(
        point - shadow_offset, shadow_center, shadow_half_size, shadow_radius, shadow_extrusion,
    );
    let contact_shadow = (1.0 - smoothstep(-0.008, 0.040, shadow_distance)) * 0.78;
    let penumbra = 0.085 + material.elevation / short_edge * 0.18;
    let soft_shadow = (1.0 - smoothstep(-0.012, penumbra, shadow_distance)) * 0.36;
    let shadow_alpha = contact_shadow + soft_shadow * (1.0 - contact_shadow);

    // Ivory body with darker vertical walls and a softly lit upper rim.
    let local_light = normalize(material.lighting.xy);
    let side_light = clamp(0.72 + dot(point, local_light) * 0.28, 0.0, 1.0);
    let porcelain_front = mix(
        vec3<f32>(0.38, 0.36, 0.28),
        vec3<f32>(0.75, 0.73, 0.62),
        side_light,
    );
    let porcelain_right = mix(
        vec3<f32>(0.65, 0.63, 0.54),
        vec3<f32>(0.94, 0.92, 0.82),
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
        vec3<f32>(0.76, 0.74, 0.64),
        vec3<f32>(0.98, 0.97, 0.88),
        side_light,
    );
    let upward_surface = max(
        max(max(max(own_hand_mode,guide_hand_mode), side_stand_mode), opposite_stand_mode),
        max(own_meld_mode, concealed_meld_mode),
    );
    let roof = upward_surface;
    let depth_color = mix(porcelain_front, porcelain_top, roof);
    let end_color = mix(porcelain_right, porcelain_front * 0.95, side_stand_mode);
    var color = mix(depth_color, end_color, lateral_weight);
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
    let body_depth = clamp(length(max(abs(face_point) - face_half_size, vec2<f32>(0.0))) / max(length(extrusion), 0.001), 0.0, 1.0);
    let side_shading = mix(1.02, 0.67, smoothstep(0.10, 1.0, body_depth));
    let roof_shading = mix(0.84, 1.10, smoothstep(0.0, 0.85, body_depth));
    color *= mix(side_shading, roof_shading, standing * roof);
    color *= 1.0 - depth_weight * 0.06;
    color += vec3<f32>(0.0, 0.018, 0.010) * depth_weight * (1.0 - upward_surface);
    // A thin resin backing is visible below the ivory body of tiles lying face-up.
    let backing_band = smoothstep(0.70, 0.87, body_depth) * (1.0 - standing) * (1.0 - wall_mode);
    color = mix(color, vec3<f32>(0.10, 0.22, 0.15) * side_shading, backing_band * 0.80);
    let own_back_rim = smoothstep(0.42,0.70,body_depth)*own_hand_mode;
    color = mix(color,vec3<f32>(0.40,0.54,0.26)*roof_shading,own_back_rim);

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
    // Sample at the visible pixel footprint so recessed strokes survive minification in the river.
    let texel = max(vec2<f32>(2.35) / dimensions, fwidth(face_uv) * 0.65);
    var height = 0.0;
    var gradient = vec2<f32>(0.0);
    var glyph = vec4<f32>(0.0);
    // Concealed hands and walls have no engraving; avoid their unnecessary texture reads.
    if material.params.y < 0.5 {
        height = height_at(face_uv);
        gradient = vec2<f32>(
            height_at(face_uv + vec2<f32>(texel.x, 0.0)) - height_at(face_uv - vec2<f32>(texel.x, 0.0)),
            height_at(face_uv - vec2<f32>(0.0, texel.y)) - height_at(face_uv + vec2<f32>(0.0, texel.y)),
        );
        glyph = sample_glyph(face_uv);
    }
    let groove_edge = clamp(length(gradient) * 3.3, 0.0, 1.0);
    // A gently crowned ceramic face carries a broad, soft reflection as well as the engraved grooves.
    let crown = face_point / face_half_size * vec2<f32>(0.20, 0.16);
    let normal = normalize(vec3<f32>(gradient.x * 3.2 + crown.x, gradient.y * 3.2 + crown.y, 1.0));
    let light = normalize(vec3<f32>(material.lighting.xy, 1.30));
    let half_vector = normalize(light + vec3<f32>(0.0, 0.0, 1.0));
    let diffuse = 0.79 + 0.21 * max(dot(normal, light), 0.0);
    let specular = pow(max(dot(normal, half_vector), 0.0), 38.0) * (0.13 + groove_edge * 0.34);
    let soft_reflection = pow(max(dot(normal, half_vector), 0.0), 14.0) * 0.13;

    let edge_depth = clamp(-face_distance / 0.075, 0.0, 1.0);
    let bevel = smoothstep(0.0, 0.95, edge_depth);
    let bevel_zone = 1.0 - bevel;
    let bevel_direction = normalize(vec3<f32>(
        corner_vector.x * 12.0,
        corner_vector.y * 12.0,
        0.65,
    ));
    let bevel_light = max(dot(bevel_direction, light), 0.0);
    let warmth = clamp(0.58 - point.y * 0.22 - point.x * 0.05, 0.0, 1.0);
    let ivory = mix(mix(vec3<f32>(0.80, 0.78, 0.69), vec3<f32>(0.98, 0.97, 0.89), warmth), vec3<f32>(0.82,0.80,0.71), own_hand_mode);
    let sheen = 1.0 - own_hand_mode * 0.72;
    let surface_variation = sin(face_uv.x * 173.0 + sin(face_uv.y * 127.0) * 2.7) * 0.003;
    var face_color = ivory * (1.0 + surface_variation) * diffuse
        * (0.965 + bevel * 0.035)
        * (1.0 - bevel_zone * 0.22 + bevel_zone * bevel_light * 0.23)
        + vec3<f32>(specular + soft_reflection + bevel_zone * bevel_light * 0.075) * sheen;
    let glaze_band = pow(
        clamp(1.0 - abs(face_uv.x * 0.62 + face_uv.y * 0.38 - 0.32) * 3.1, 0.0, 1.0),
        6.0,
    ) * smoothstep(0.02, 0.16, face_uv.x)
        * smoothstep(0.02, 0.16, face_uv.y)
        * smoothstep(0.02, 0.16, 1.0 - face_uv.x)
        * smoothstep(0.02, 0.16, 1.0 - face_uv.y);
    face_color += vec3<f32>(glaze_band * 0.10 * sheen);
    let polished_edge = (1.0 - smoothstep(0.006, 0.020, abs(face_distance + 0.018)))
        * bevel_light;
    face_color += vec3<f32>(polished_edge * 0.16 * sheen);

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
    face_color += vec3<f32>(specular * (1.0 - glyph.a * 0.70) * sheen);

    let back_mode = step(0.5, material.params.y);
    let back_inset_half_size = face_half_size - vec2<f32>(0.035);
    let back_inset_distance = rounded_box(face_point, back_inset_half_size, 0.060);
    let back_inset = mask_from_distance(back_inset_distance);
    let back_rim = clamp(1.0 - abs(back_inset_distance) / 0.030, 0.0, 1.0);
    let back_pattern = 0.5 + sin(face_uv.x * 64.0 + sin(face_uv.y * 5.0) * 0.4) * 0.015;
    let back_base = mix(
        vec3<f32>(0.025, 0.145, 0.090),
        vec3<f32>(0.120, 0.345, 0.220),
        0.42 + back_pattern * 0.45,
    );
    let back_light = clamp(0.88 + dot(face_point / face_half_size, local_light) * 0.23, 0.57, 1.17);
    var back_color = mix(ivory * 0.92, back_base * back_light, back_inset);
    let back_glaze = pow(clamp(1.0 - abs(face_uv.x - 0.26) * 1.6, 0.0, 1.0), 5.0) * back_inset;
    back_color += vec3<f32>(back_rim * 0.045 + specular * 0.16 + soft_reflection * 0.16 + back_glaze * 0.025);
    face_color = mix(face_color, back_color, back_mode);

    let interaction_amount = step(0.5, material.params.x);
    let pressed = step(1.5, material.params.x);
    let interactive = mix(face_color, face_color * vec3<f32>(0.90, 0.98, 0.92), 0.18 + pressed * 0.18)
        + vec3<f32>(0.035, 0.055, 0.025) * (1.0 - pressed);
    face_color = mix(face_color, interactive, interaction_amount);
    let has_highlight = material.highlight.w > 0.0;
    let overlay_color = select(vec3<f32>(0.39, 0.76, 1.0), material.highlight.rgb, has_highlight);
    let overlay_strength = select(clamp(material.lighting.z, 0.0, 1.0) * 0.52, material.highlight.w, has_highlight);
    face_color = mix(face_color, overlay_color, clamp(overlay_strength, 0.0, 1.0));
    color = mix(color, face_color, face);

    let alpha = tile_alpha + shadow_alpha * (1.0 - tile_alpha);
    let shadow_color = vec3<f32>(0.004, 0.008, 0.006);
    let composed = (color * tile_alpha + shadow_color * shadow_alpha * (1.0 - tile_alpha)) / max(alpha, 0.0001);
    return vec4<f32>(composed, alpha * material.params.z);
}
