use super::state::{Network, Operation};
use ndarray::{Array2, ArrayView2};

pub(super) fn run(network: &Network, input: &[f32], batch: usize) -> Result<Vec<f32>, String> {
    let mut slots = vec![input.to_vec()];
    for step in &network.steps {
        let input = &slots[step.input];
        let output = match &step.operation {
            Operation::Linear { weight, bias, relu } => {
                let [outputs, inputs] = weight.shape.as_slice() else {
                    unreachable!()
                };
                let x =
                    ArrayView2::from_shape((batch, *inputs), input).map_err(|e| e.to_string())?;
                let w = ArrayView2::from_shape((*outputs, *inputs), weight.values.as_ref())
                    .map_err(|e| e.to_string())?;
                let mut y = x.dot(&w.t());
                for mut row in y.rows_mut() {
                    for (value, bias) in row.iter_mut().zip(bias.values.iter()) {
                        *value += bias;
                        if *relu {
                            *value = value.max(0.0);
                        }
                    }
                }
                y.into_raw_vec_and_offset().0
            }
            Operation::Conv3x3 { weight, bias, relu } => {
                let [channels, height, width] = network.shape(step.input) else {
                    unreachable!()
                };
                let outputs = weight.shape[0];
                let area = height * width;
                let kernel = channels * 9;
                let w = ArrayView2::from_shape((outputs, kernel), weight.values.as_ref())
                    .map_err(|e| e.to_string())?;
                let mut output = vec![0.0; batch * outputs * area];
                for n in 0..batch {
                    let cols = Array2::from_shape_fn((area, kernel), |(position, k)| {
                        let x = position % width;
                        let y = position / width;
                        let dx = k % 3;
                        let dy = k / 3 % 3;
                        let c = k / 9;
                        if x + dx == 0 || y + dy == 0 || x + dx > *width || y + dy > *height {
                            0.0
                        } else {
                            input
                                [n * channels * area + c * area + (y + dy - 1) * width + x + dx - 1]
                        }
                    });
                    let result = cols.dot(&w.t());
                    for c in 0..outputs {
                        for p in 0..area {
                            let value = result[(p, c)] + bias.values[c];
                            output[n * outputs * area + c * area + p] =
                                if *relu { value.max(0.0) } else { value };
                        }
                    }
                }
                output
            }
            Operation::AddRelu { other } => input
                .iter()
                .zip(&slots[*other])
                .map(|(x, y)| (x + y).max(0.0))
                .collect(),
        };
        slots.push(output);
    }
    let output = slots.swap_remove(network.output);
    if output.iter().any(|n| !n.is_finite()) {
        return Err("模型输出包含非有限数值".into());
    }
    Ok(output)
}
