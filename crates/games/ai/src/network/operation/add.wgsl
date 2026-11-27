@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= p.length { return; }
    var value = 0.0;
        value = x[index] + residual[index];
    
    if p.relu != 0u { value = max(value, 0.0); }
    y[index] = value;
}
