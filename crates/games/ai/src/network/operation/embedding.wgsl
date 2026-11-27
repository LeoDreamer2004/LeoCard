@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= p.length { return; }
    var value = 0.0;
        let row = index / p.outputs;
        let column = index % p.outputs;
        value = weights[u32(x[row]) * p.outputs + column] + bias[(p.height - p.batch + row) * p.outputs + column];
    
    if p.relu != 0u { value = max(value, 0.0); }
    y[index] = value;
}
