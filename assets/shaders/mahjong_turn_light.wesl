import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct MahjongTurnLightMaterial {
    params: vec4<f32>,
    color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: MahjongTurnLightMaterial;
@group(1) @binding(1) var panel_texture: texture_2d<f32>;
@group(1) @binding(2) var panel_sampler: sampler;

fn segment_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
    let axis = end - start;
    let fraction = clamp(dot(point - start, axis) / dot(axis, axis), 0.0, 1.0);
    return length(point - start - axis * fraction);
}

fn panel_sample(point: vec2<f32>) -> vec4<f32> {
    let ratio=352.0/390.0;
    let v=point.y/(ratio+(1.0-ratio)*point.y);
    let u=0.5+(point.x-0.5)*(1.0-(1.0-ratio)*v)/ratio;
    let valid=step(0.0,u)*step(u,1.0)*step(0.0,v)*step(v,1.0);
    let color=textureSample(panel_texture,panel_sampler,clamp(vec2<f32>(u,v),vec2<f32>(0.0),vec2<f32>(1.0)));
    return vec4<f32>(color.rgb,color.a*valid);
}

fn console_surface(uv: vec2<f32>) -> vec4<f32> {
    let point=uv*vec2<f32>(1.12,1.28)-vec2<f32>(0.06,0.08);
    let roof=panel_sample(point);
    let depth=vec2<f32>(0.002,0.035);
    let base=panel_sample(point-depth);
    let middle=panel_sample(point-depth*0.5);
    let side=max(base.a,middle.a)*(1.0-roof.a);
    let body_alpha=roof.a+side;
    let shadow_center=point-depth-vec2<f32>(0.006,0.014);
    let contact=panel_sample(shadow_center).a*0.48;
    let penumbra=(
        panel_sample(shadow_center+vec2<f32>(0.008,0.012)).a+
        panel_sample(shadow_center+vec2<f32>(-0.008,0.012)).a+
        panel_sample(shadow_center+vec2<f32>(0.008,-0.012)).a+
        panel_sample(shadow_center+vec2<f32>(-0.008,-0.012)).a
    )*0.065;
    let shadow=max(contact,penumbra)*(1.0-body_alpha);
    let side_color=vec3<f32>(0.065,0.070,0.080)+base.rgb*0.12;
    let alpha=body_alpha+shadow;
    let premultiplied=roof.rgb*roof.a+side_color*side+vec3<f32>(0.005,0.008,0.007)*shadow;
    return vec4<f32>(premultiplied/max(alpha,0.0001),alpha);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    if material.params.w > 0.5 {
        return console_surface(in.uv);
    }
    let ratio=352.0/390.0;
    let v=in.uv.y/(ratio+(1.0-ratio)*in.uv.y);
    let u=0.5+(in.uv.x-0.5)*(1.0-(1.0-ratio)*v)/ratio;
    let uv=vec2<f32>(u,v);
    var start = vec2<f32>(0.37, 0.875);
    var end = vec2<f32>(0.63, 0.875);
    if material.params.x > 0.5 && material.params.x < 1.5 {
        start = vec2<f32>(0.885, 0.36);
        end = vec2<f32>(0.885, 0.63);
    } else if material.params.x > 1.5 && material.params.x < 2.5 {
        start = vec2<f32>(0.37, 0.12);
        end = vec2<f32>(0.63, 0.12);
    } else if material.params.x > 2.5 {
        start = vec2<f32>(0.115, 0.36);
        end = vec2<f32>(0.115, 0.63);
    }
    let distance = segment_distance(uv * in.size, start * in.size, end * in.size);
    let unit = min(in.size.x, in.size.y);
    let core = 1.0 - smoothstep(unit * 0.004, unit * 0.012, distance);
    let glow = exp(-distance * distance / max(unit * unit * 0.00075, 0.001));
    let intensity = material.params.y;
    let alpha = max(core * 0.96, glow * 0.72) * intensity;
    let color = mix(material.color.rgb, vec3<f32>(1.0, 0.74, 0.35), core * 0.60);
    return vec4<f32>(color, alpha);
}
