@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= p.length { return; }
    var value = 0.0;
        let row = index / p.outputs;
        let output = index % p.outputs;
        value = bias[output];
        for (var i = 0u; i < p.inputs; i += 1u) {
            value += x[row * p.inputs + i] * weights[output * p.inputs + i];
        }
    
    if p.relu != 0u { value = max(value, 0.0); }
    y[index] = value;
}
