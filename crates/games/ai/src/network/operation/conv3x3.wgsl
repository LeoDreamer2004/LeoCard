@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= p.length { return; }
    var value = 0.0;
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
    
    if p.relu != 0u { value = max(value, 0.0); }
    y[index] = value;
}
