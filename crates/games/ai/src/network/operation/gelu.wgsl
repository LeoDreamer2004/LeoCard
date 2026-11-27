@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= p.length { return; }
    var value = 0.0;
        let z = x[index] * 0.7071067811865476;
        let t = 1.0 / (1.0 + 0.3275911 * abs(z));
        let polynomial = (((((1.0614054 * t - 1.4531521) * t) + 1.4214138) * t - 0.28449672) * t + 0.2548296) * t;
        value = 0.5 * x[index] * (1.0 + sign(z) * (1.0 - polynomial * exp(-z * z)));
    
    if p.relu != 0u { value = max(value, 0.0); }
    y[index] = value;
}
