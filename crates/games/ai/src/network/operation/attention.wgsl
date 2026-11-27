fn attention_score(row: u32, key: u32, start: u32, width: u32) -> f32 {
    var score = 0.0;
    for (var i = 0u; i < width; i += 1u) {
        score += x[row * 3u * p.outputs + start + i] * x[key * 3u * p.outputs + p.outputs + start + i];
    }
    return score * inverseSqrt(f32(width));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= p.length { return; }
    var value = 0.0;
        let row = index / p.outputs;
        let column = index % p.outputs;
        let width = p.outputs / p.height;
        let start = (column / width) * width;
        var maximum = -3.4e38;
        for (var key = 0u; key <= row; key += 1u) {
            maximum = max(maximum, attention_score(row, key, start, width));
        }
        var total = 0.0;
        for (var key = 0u; key <= row; key += 1u) {
            let weight = exp(attention_score(row, key, start, width) - maximum);
            total += weight;
            value += weight * x[key * 3u * p.outputs + 2u * p.outputs + column];
        }
        value /= total;
    if p.relu != 0u { value = max(value, 0.0); }
    y[index] = value;
}
