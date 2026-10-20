import bevy_ui_render::ui_vertex_output::UiVertexOutput;
import bevy_render::globals::Globals;

@group(0) @binding(1) var<uniform> globals: Globals;

struct GraphCurveMaterial {
    start_control: vec4<f32>,
    end_size: vec4<f32>,
    color: vec4<f32>,
    appearance: vec4<f32>,
}
@group(1) @binding(0) var<uniform> material: GraphCurveMaterial;

fn segment(point: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let delta = b - a;
    let t = clamp(dot(point - a, delta) / max(dot(delta, delta), 0.001), 0.0, 1.0);
    return vec2<f32>(length(point - a - t * delta), t);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let point = in.uv * material.end_size.zw;
    let start = material.start_control.xy;
    let control = material.start_control.zw;
    let end = material.end_size.xy;
    var previous = start;
    var distance = 10000.0;
    var progress = 0.0;
    for (var i = 1u; i <= 24u; i += 1u) {
        let t = f32(i) / 24.0;
        let pos = (1.0-t)*(1.0-t)*start + 2.0*(1.0-t)*t*control + t*t*end;
        let result = segment(point, previous, pos);
        if result.x < distance { distance = result.x; progress = (f32(i-1u)+result.y)/24.0; }
        previous = pos;
    }
    let activated = material.appearance.x;
    let highlight = material.appearance.y;
    let directions = material.appearance.z;
    let time = globals.time;
    let path_length = length(start-control) + length(control-end);
    let dash_phase = fract(progress * path_length / 11.0);
    let dash = select(select(0.0, 1.0, dash_phase < 0.58), 1.0, activated > 0.5);
    var arrow = 10000.0;
    if directions > 0.5 {
        let tangent = normalize(end-control+vec2<f32>(0.001));
        let normal = vec2<f32>(-tangent.y, tangent.x);
        arrow = min(segment(point,end,end-tangent*9.0+normal*4.0).x,segment(point,end,end-tangent*9.0-normal*4.0).x);
    }
    if directions > 1.5 {
        let tangent = normalize(start-control+vec2<f32>(0.001));
        let normal = vec2<f32>(-tangent.y, tangent.x);
        arrow = min(arrow,min(segment(point,start,start-tangent*9.0+normal*4.0).x,segment(point,start,start-tangent*9.0-normal*4.0).x));
    }
    let width = 0.65 + activated*0.25 + highlight*0.4;
    let aa = max(fwidth(distance),0.6);
    let body = (1.0-smoothstep(width-aa,width+aa,distance))*dash;
    let head = 1.0-smoothstep(width-aa,width+aa,arrow);
    let glow = exp(-distance*distance/20.0)*highlight*0.12;
    let flow = exp(-pow((progress-fract(time*0.18))*22.0,2.0))*highlight*activated*0.16;
    let opacity = max(body,head)*mix(0.7,0.98,max(activated,highlight)) + glow + flow*body;
    return vec4<f32>(material.color.rgb,clamp(opacity,0.0,1.0)*material.color.a);
}
