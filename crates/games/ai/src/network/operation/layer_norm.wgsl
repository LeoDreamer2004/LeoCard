@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= p.length { return; }
    var value = 0.0;
        let start = (index / p.outputs) * p.outputs;
        var mean = 0.0;
        for (var i = 0u; i < p.outputs; i += 1u) { mean += x[start + i]; }
        mean /= f32(p.outputs);
        var variance = 0.0;
        for (var i = 0u; i < p.outputs; i += 1u) {
            let delta = x[start + i] - mean;
            variance += delta * delta;
        }
        variance /= f32(p.outputs);
        let column = index % p.outputs;
        value = (x[index] - mean) * inverseSqrt(variance + 0.00001) * weights[column] + bias[column];
    
    if p.relu != 0u { value = max(value, 0.0); }
    y[index] = value;
}
