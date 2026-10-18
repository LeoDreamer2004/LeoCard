import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct UnoDragGuideMaterial {
    start_control: vec4<f32>,
    end_size: vec4<f32>,
    color: vec4<f32>,
    phase: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: UnoDragGuideMaterial;

fn segment_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> vec2<f32> {
    let delta = end - start;
    let fraction = clamp(dot(point - start, delta) / max(dot(delta, delta), 0.001), 0.0, 1.0);
    return vec2<f32>(length(point - start - fraction * delta), fraction);
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
    for (var index = 1u; index <= 32u; index += 1u) {
        let t = f32(index) / 32.0;
        let position = (1.0 - t) * (1.0 - t) * start + 2.0 * (1.0 - t) * t * control + t * t * end;
        let sample = segment_distance(point, previous, position);
        if sample.x < distance {
            distance = sample.x;
            progress = (f32(index - 1u) + sample.y) / 32.0;
        }
        previous = position;
    }
    let tangent = normalize(end - control + vec2<f32>(0.001));
    let normal = vec2<f32>(-tangent.y, tangent.x);
    let arrow_left = segment_distance(point, end, end - tangent * 13.0 + normal * 6.5).x;
    let arrow_right = segment_distance(point, end, end - tangent * 13.0 - normal * 6.5).x;
    let stroke_distance = min(distance, min(arrow_left, arrow_right));
    let antialias = max(fwidth(stroke_distance), 0.65);
    let stroke = 1.0 - smoothstep(1.3 - antialias, 1.3 + antialias, stroke_distance);
    let glow = exp(-stroke_distance * stroke_distance / 23.0) * 0.24;
    let spark_position = fract(material.phase.x * 0.65);
    let spark = exp(-pow((progress - spark_position) * 24.0, 2.0)) * exp(-distance * distance / 7.0);
    let color = mix(material.color.rgb * 0.68, vec3<f32>(1.0), stroke * 0.38 + spark * 0.45);
    let fade = mix(0.25, 1.0, smoothstep(0.0, 0.22, progress));
    return vec4<f32>(color, clamp(stroke * 0.86 + glow + spark * 0.24, 0.0, 1.0) * fade * material.color.a);
}
