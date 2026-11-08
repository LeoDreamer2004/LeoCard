import bevy_render::globals::Globals;
import bevy_ui_render::ui_vertex_output::UiVertexOutput;
import super::mahjong_tile_bindings::{material, sample_glyph, height_at, height_texture};
import super::mahjong_tile_geometry::{box_intersection, swept_rounded_box_distance};
@group(0) @binding(1) var<uniform> globals: Globals;

fn projected_ground_shadow(origin: vec3<f32>, ray: vec3<f32>, half: vec3<f32>) -> vec4<f32> {
    let ground_distance = (-material.ray_origin.w - origin.y) / ray.y;
    let ground = origin + ray * ground_distance;
    let elevation = max(material.ray_origin.w - half.y, 0.0);
    let direction = -material.lighting.xy / 2.3;
    let distance = swept_rounded_box_distance(
        ground.xz, direction * elevation, half.xz, material.box_half_size.w, direction * (half.y * 2.0),
    );
    let contact = (1.0 - smoothstep(-0.01, 0.045, distance)) * 0.56;
    let soft = (1.0 - smoothstep(-0.01, 0.14, distance)) * 0.26;
    return vec4<f32>(0.005, 0.009, 0.007, (contact + soft * (1.0 - contact)) * material.params.z);
}

fn projected_tile(in: UiVertexOutput) -> vec4<f32> {
    let origin = material.ray_origin.xyz;
    let ray = material.ray_forward.xyz + material.ray_right.xyz * (in.uv.x-0.5) + material.ray_down.xyz * (in.uv.y-0.5);
    let half = material.box_half_size.xyz;
    if material.shadow_only > 0.5 {
        return projected_ground_shadow(origin, ray, half);
    }
    let bevel = material.box_half_size.w;
    let hit = box_intersection(origin, ray, half, bevel);
    let gap = hit.near - hit.far;
    let aa = clamp(fwidth(gap) * 0.6, 0.0001, 0.04);
    let coverage = (1.0-smoothstep(-aa, aa, gap)) * step(0.0, hit.near);
    let point = origin + ray * hit.near;
    let edge = max(abs(point) - half + vec3<f32>(bevel), vec3<f32>(0.0));
    let normal = normalize(edge * sign(point) + hit.normal * 0.00001);
    let light = normalize(vec3<f32>(material.lighting.x, 2.3, material.lighting.y));
    let view = normalize(-ray);
    let half_vector = normalize(light + view);
    let diffuse = 0.53 + max(dot(normal, light), 0.0) * 0.47;
    let ivory = vec3<f32>(0.82, 0.80, 0.71);
    let reflection = pow(max(dot(normal, half_vector), 0.0), 36.0) * 0.20;
    var color = ivory * diffuse + vec3<f32>(reflection);
    let upright = material.params.w < -1.5 && material.params.w > -3.5;
    let decorated = select(step(0.92, hit.normal.y), step(0.92, hit.normal.z), upright);
    let uv = select(
        vec2<f32>(point.x / (2.0*half.x)+0.5, 0.5-point.z/(2.0*half.z)),
        vec2<f32>(point.x / (2.0*half.x)+0.5, 0.5-point.y/(2.0*half.y)), upright,
    );
    let crown = (uv-vec2<f32>(0.5))*vec2<f32>(0.10,0.08);
    var pigment = vec4<f32>(0.0); var groove = vec2<f32>(0.0); var depth = 0.0;
    if material.params.y < 0.5 {
        pigment = sample_glyph(uv);
        let texel = max(vec2<f32>(2.0)/vec2<f32>(textureDimensions(height_texture)), fwidth(uv)*0.55);
        depth = height_at(uv);
        groove = vec2<f32>(height_at(uv+vec2<f32>(texel.x,0.0))-height_at(uv-vec2<f32>(texel.x,0.0)),
            height_at(uv-vec2<f32>(0.0,texel.y))-height_at(uv+vec2<f32>(0.0,texel.y)));
    }
    let face_normal = normalize(select(vec3<f32>(groove.x*2.5+crown.x,1.0,-groove.y*2.5+crown.y),
        vec3<f32>(groove.x*2.5+crown.x,groove.y*2.5+crown.y,1.0),upright));
    let face_diffuse = 0.74 + max(dot(face_normal,light),0.0)*0.26;
    let face_gloss = pow(max(dot(face_normal,half_vector),0.0),44.0)*0.18;
    var face_color = ivory*face_diffuse + vec3<f32>(face_gloss);
    let groove_rim = clamp(length(groove)*2.2,0.0,1.0);
    face_color *= 1.0-depth*0.07;
    let ink = pigment.rgb*(0.91+face_diffuse*0.09) + vec3<f32>(groove_rim*face_gloss*0.22);
    face_color = mix(face_color,ink,pigment.a);
    if material.params.y > 0.5 {
        let inset = smoothstep(0.015,0.040,min(min(uv.x,uv.y),min(1.0-uv.x,1.0-uv.y)));
        let gradient = clamp(0.89+dot(uv-vec2<f32>(0.5),vec2<f32>(-0.10,-0.22)),0.70,1.04);
        let resin = vec3<f32>(0.33,0.46,0.21)*gradient*(0.82+max(dot(face_normal,light),0.0)*0.18);
        face_color = mix(ivory*face_diffuse,resin+vec3<f32>(face_gloss*0.30),inset);
    }
    if upright {
        // The colored resin back wraps over the inner strip of the white roof and the exposed end.
        let back_depth=select(-point.z,point.z,material.params.y>0.5);
        let resin_depth=select(0.22,0.28,abs(material.params.w+3.0)<0.1);
        let laminate=smoothstep(half.z-resin_depth,half.z-resin_depth+0.04,back_depth);
        color=mix(color,vec3<f32>(0.40,0.54,0.26)*diffuse+vec3<f32>(reflection*0.4),laminate);
    }
    if !upright {
        // A real resin layer occupies the lower third of the full tile, beneath the ivory body.
        let layer_height = half.y * 2.0;
        let layer_y = (point.y + half.y) / layer_height;
        var tile_y = clamp(layer_y, 0.0, 1.0);
        if abs(material.params.w-2.0)<0.1 {
            tile_y = clamp(layer_y * 2.0 - step(0.0, point.y), 0.0, 1.0);
        }
        let from_back = select(tile_y, 1.0-tile_y, material.params.y>0.5);
        let backing = 1.0 - smoothstep(0.31, 0.35, from_back);
        color = mix(color, vec3<f32>(0.40,0.54,0.26)*diffuse+vec3<f32>(reflection*0.4), backing);
        if abs(material.params.w-2.0)<0.1 {
            let seam=1.0-smoothstep(0.004,0.018,abs(point.y));
            color *= 1.0-seam*0.25;
        }
    }
    let overlay_strength = max(clamp(material.lighting.z,0.0,1.0)*0.52,material.highlight.w);
    let overlay_color=select(vec3<f32>(0.39,0.76,1.0),material.highlight.rgb,material.highlight.w>0.0);
    face_color=mix(face_color,overlay_color,overlay_strength);
    color=mix(color,face_color,decorated);
    var alpha=coverage;
    var premultiplied=color*coverage;
    if material.response.z>0.0 {
        let breath=0.80+0.20*sin(globals.time*2.617994);
        let outline=exp(-abs(gap)/max(aa*2.2,0.0001))*breath;
        let gold=vec3<f32>(1.0,0.67,0.08);
        premultiplied=mix(premultiplied,gold,outline*0.9);
        alpha=max(alpha,outline);
        let bob=(0.5+0.5*sin(globals.time*4.188790))*6.0/14.0;
        let arrow=(in.uv-material.response.xy)/material.response.zw+vec2<f32>(0.0,bob);
        let distance=max(max(abs(arrow.x)+arrow.y,arrow.y),-1.0-arrow.y);
        let arrow_alpha=(1.0-smoothstep(-0.035,0.035,distance));
        premultiplied=mix(premultiplied,vec3<f32>(0.89,0.068,0.0035),arrow_alpha);
        alpha=alpha*(1.0-arrow_alpha)+arrow_alpha;
    }
    return vec4<f32>(premultiplied/max(alpha,0.0001),alpha*material.params.z);
}
