import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct GraphNodeMaterial {
    color: vec4<f32>, state: vec4<f32>, sprite: vec4<f32>, activity: vec4<f32>
}
@group(1) @binding(0) var<uniform> material: GraphNodeMaterial;
@group(1) @binding(1) var node_icon: texture_2d<f32>;
@group(1) @binding(2) var node_sampler: sampler;

fn dark_surface() -> vec3<f32> {
    let gray = vec3<f32>(0.0176,0.0194,0.0307);
    let luminance = vec3<f32>(0.2126,0.7152,0.0722);
    let category = material.color.rgb * dot(gray,luminance)
        / max(dot(material.color.rgb,luminance),0.001);
    return mix(gray,category,select(0.45,0.25,material.state.w > 0.5));
}

fn sprite_pixel(uv: vec2<f32>) -> vec4<f32> {
    if any(uv < vec2<f32>(0.0)) || any(uv >= vec2<f32>(1.0)) {
        return vec4<f32>(0.0);
    }
    // Sample texel centers and stay inside this sprite, never an adjacent atlas cell.
    let pixel = floor(uv * material.sprite.zw) + vec2<f32>(0.5);
    let source = (material.sprite.xy + pixel) / vec2<f32>(textureDimensions(node_icon));
    return textureSampleLevel(node_icon,node_sampler,source,0.0);
}

fn expanded_alpha(uv: vec2<f32>, radius: f32) -> f32 {
    let directions = array<vec2<f32>,8>(
        vec2<f32>(1.0,0.0),vec2<f32>(-1.0,0.0),
        vec2<f32>(0.0,1.0),vec2<f32>(0.0,-1.0),
        vec2<f32>(0.7071,0.7071),vec2<f32>(-0.7071,0.7071),
        vec2<f32>(0.7071,-0.7071),vec2<f32>(-0.7071,-0.7071)
    );
    var alpha = sprite_pixel(uv).a;
    for (var i = 0u; i < 8u; i += 1u) {
        alpha = max(alpha,sprite_pixel(uv + directions[i] * radius / 64.0).a);
    }
    return alpha;
}

fn with_wave(color: vec3<f32>, alpha: f32, wave: f32, outline: vec3<f32>) -> vec4<f32> {
    let pulse = wave * (1.0-alpha);
    let combined = alpha + pulse;
    return vec4<f32>((color*alpha + outline*pulse)
        / max(combined,0.001),combined);
}

fn with_revealed_backdrop(foreground: vec4<f32>, point: vec2<f32>) -> vec4<f32> {
    let distance = length(point)-36.0;
    let aa = max(fwidth(distance),0.55);
    let coverage = (1.0-smoothstep(-aa,aa,distance))
        * clamp(material.state.z,0.0,1.0) * 0.78;
    let background = coverage*(1.0-foreground.a);
    let alpha = foreground.a+background;
    let gray = vec3<f32>(0.030,0.032,0.038);
    return vec4<f32>((foreground.rgb*foreground.a+gray*background)
        / max(alpha,0.001),alpha);
}

fn with_combo_halo(foreground: vec4<f32>, point: vec2<f32>) -> vec4<f32> {
    if material.activity.z <= 0.001 {
        return foreground;
    }
    let breath = 0.5-0.5*cos(material.activity.w*6.283185/2.4);
    let strength = material.activity.z*(0.45+0.55*breath);
    let distance = length(point)-(39.0+1.5*breath);
    let aa = max(fwidth(distance),0.55);
    let ring = 1.0-smoothstep(1.0-aa,1.0+aa,abs(distance));
    let glow = exp(-pow(distance/4.0,2.0))*0.28;
    let halo = clamp((ring*0.78+glow)*strength,0.0,1.0)*(1.0-foreground.a);
    let alpha = foreground.a+halo;
    let gold = vec3<f32>(1.0,0.58,0.065);
    return vec4<f32>((foreground.rgb*foreground.a+gold*halo)/max(alpha,0.001),alpha);
}

// Two pulses per cycle: a strong leading wave and a softer trailing wave.
// The rest of the cycle stays quiet before the next pair begins.
fn wave_phase(delay: f32) -> f32 {
    return (material.activity.y % 2.4 - delay) / 1.6;
}

fn wave_strength(phase: f32) -> f32 {
    if phase < 0.0 || phase >= 1.0 {
        return 0.0;
    }
    return pow(1.0-phase,1.4) * material.activity.x;
}

fn tag(point: vec2<f32>) -> vec4<f32> {
    let distance = length(point) - (20.0 + material.state.x*1.2);
    let aa = max(fwidth(distance),0.55);
    let fill = 1.0-smoothstep(-aa,aa,distance);
    let selected = material.state.y;
    let revealed = material.state.z;
    let width = 1.1 + selected*1.2 + revealed*0.3;
    var ring = 1.0-smoothstep(width-aa,width+aa,abs(distance));
    if revealed < 0.5 {
        let angle = atan2(point.y,point.x);
        ring *= select(0.0,1.0,fract((angle+3.141593)/6.283185*14.0)<0.6);
    }
    let tint = mix(material.color.rgb,vec3<f32>(0.674,0.587,0.296),selected);
    let emphasis = max(material.state.x,selected);
    let halo = exp(-pow(distance/3.4,2.0))*emphasis*0.15;
    let alpha = clamp(max(fill,ring*material.color.a) + halo*material.color.a,0.0,1.0);
    let rgb = mix(dark_surface(),tint,ring*mix(0.55,1.0,max(emphasis,revealed))*material.color.a);
    var wave = 0.0;
    for (var i = 0u; i < 2u; i += 1u) {
        let phase = wave_phase(f32(i)*0.4);
        let strength = select(0.75,0.38,i == 1u);
        let pulse = (1.0-smoothstep(0.6,1.7,abs(distance-(3.0+phase*14.0))))
            * wave_strength(phase) * strength;
        wave = max(wave,pulse);
    }
    return with_wave(rgb,alpha,wave,tint);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let point = (in.uv-vec2<f32>(0.5))*96.0;
    if material.state.w > 0.5 {
        return tag(point);
    }
    let uv = point/64.0 + vec2<f32>(0.5);
    let pixel = sprite_pixel(uv);
    let activation = clamp(material.state.x,0.0,1.0);
    let selected = clamp(material.state.y,0.0,1.0);
    let luminance = dot(pixel.rgb,vec3<f32>(0.2126,0.7152,0.0722));
    // Retain facial features and block faces beneath the subdued silhouette mask.
    let muted = dark_surface() * mix(0.20,2.50,pow(luminance,0.45));
    let color = mix(muted,pixel.rgb,activation);
    // Alpha dilation traces the actual transparent image boundary, including holes.
    // Hover only restores color; selected alone enables the outline and its glow.
    var edge = 0.0;
    var glow = 0.0;
    if selected > 0.001 {
        edge = max(expanded_alpha(uv,1.6)-pixel.a,0.0)*selected;
        glow = max(expanded_alpha(uv,4.0)-pixel.a,0.0)*selected*0.20;
    }
    let outline = mix(material.color.rgb,vec3<f32>(1.0),0.25);
    let border = edge*(1.0-pixel.a);
    let halo = glow*(1.0-pixel.a)*(1.0-edge);
    let alpha = clamp(pixel.a+border+halo,0.0,1.0);
    let rgb = (color*pixel.a + outline*(border+halo))/max(alpha,0.001);
    var wave = 0.0;
    if material.activity.x > 0.001 {
        for (var i = 0u; i < 2u; i += 1u) {
            let phase = wave_phase(f32(i)*0.4);
            let strength = wave_strength(phase) * select(0.65,0.33,i == 1u);
            if strength > 0.0 {
                let pulse = max(expanded_alpha(uv,3.0+phase*11.0)
                    -expanded_alpha(uv,1.0+phase*11.0),0.0) * strength;
                wave = max(wave,pulse);
            }
        }
    }
    return with_combo_halo(with_revealed_backdrop(with_wave(rgb,alpha,wave,outline),point),point);
}
