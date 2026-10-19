import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct GraphNodeMaterial { color: vec4<f32>, state: vec4<f32> }
@group(1) @binding(0) var<uniform> material: GraphNodeMaterial;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let point = (in.uv-vec2<f32>(0.5))*64.0;
    let radius = length(point);
    let highlight = material.state.x;
    let selected = material.state.y;
    let revealed = material.state.z;
    let is_tag = material.state.w;
    let aa = max(fwidth(radius),0.55);
    let outer = 26.0 + highlight*1.2;
    let fill = 1.0-smoothstep(outer-aa,outer+aa,radius);
    let width = 1.1 + selected*1.2 + revealed*0.3;
    var ring = 1.0-smoothstep(width-aa,width+aa,abs(radius-outer));
    if is_tag > 0.5 && revealed < 0.5 {
        let angle = atan2(point.y,point.x);
        ring *= select(0.0,1.0,fract((angle+3.141593)/6.283185*14.0)<0.6);
    }
    let emphasis = max(highlight,selected);
    let tint = mix(material.color.rgb,vec3<f32>(0.674,0.587,0.296),selected);
    // Linear RGB for the former canvas surface, sRGB #242631.
    let base = vec3<f32>(0.0176,0.0194,0.0307);
    let halo = exp(-pow((radius-outer)/3.4,2.0))*emphasis*0.15;
    let alpha = max(fill,ring) + halo;
    let rgb = mix(base,tint,ring*mix(0.55,1.0,max(emphasis,revealed)));
    return vec4<f32>(rgb,clamp(alpha,0.0,1.0)*material.color.a);
}
