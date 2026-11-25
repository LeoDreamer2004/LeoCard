struct Parameters {
    kind: u32,
    inputs: u32,
    outputs: u32,
    height: u32,
    width: u32,
    batch: u32,
    relu: u32,
    length: u32,
}

@group(0) @binding(0) var<storage, read> x: array<f32>;
@group(0) @binding(1) var<storage, read> weights: array<f32>;
@group(0) @binding(2) var<storage, read> bias: array<f32>;
@group(0) @binding(3) var<storage, read> residual: array<f32>;
@group(0) @binding(4) var<storage, read_write> y: array<f32>;
@group(0) @binding(5) var<uniform> p: Parameters;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= p.length { return; }
    var value = 0.0;
    if p.kind == 0u {
        let row = index / p.outputs;
        let output = index % p.outputs;
        value = bias[output];
        for (var i = 0u; i < p.inputs; i += 1u) {
            value += x[row * p.inputs + i] * weights[output * p.inputs + i];
        }
    } else if p.kind == 1u {
        let area = p.height * p.width;
        let position = index % area;
        let output = (index / area) % p.outputs;
        let batch = index / (area * p.outputs);
        let row = i32(position / p.width);
        let column = i32(position % p.width);
        value = bias[output];
        for (var c = 0u; c < p.inputs; c += 1u) {
            for (var dy = 0u; dy < 3u; dy += 1u) {
                for (var dx = 0u; dx < 3u; dx += 1u) {
                    let sy = row + i32(dy) - 1;
                    let sx = column + i32(dx) - 1;
                    if sy >= 0 && sx >= 0 && sy < i32(p.height) && sx < i32(p.width) {
                        let xi = (batch * p.inputs + c) * area + u32(sy) * p.width + u32(sx);
                        let wi = (output * p.inputs + c) * 9u + dy * 3u + dx;
                        value += x[xi] * weights[wi];
                    }
                }
            }
        }
    } else {
        value = x[index] + residual[index];
    }
    if p.relu != 0u { value = max(value, 0.0); }
    y[index] = value;
}
