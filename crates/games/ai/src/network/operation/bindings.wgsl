struct Parameters {
    reserved: u32,
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

