//! DouZero LSTM history encoding. Dense layers use the shared CPU/GPU backend.
use leocard_ai::{DevicePreference, Network, NetworkBuilder, TensorMap};
use ndarray::{Array1, ArrayView2};

pub(crate) struct Model<'a> {
    input: ArrayView2<'a, f32>,
    recurrent: ArrayView2<'a, f32>,
    bias: Array1<f32>,
}

impl<'a> Model<'a> {
    pub(crate) fn read(tensors: &'a TensorMap) -> Result<Self, String> {
        let matrix = |name: &str, rows, columns| {
            let values = tensors
                .get(name)
                .ok_or_else(|| format!("DouZero 缺少 {name}"))?
                .expect_shape(&[rows, columns])?;
            ArrayView2::from_shape((rows, columns), values).map_err(|e| e.to_string())
        };
        let input = matrix("lstm.weight_ih_l0", 512, 162)?;
        let recurrent = matrix("lstm.weight_hh_l0", 512, 128)?;
        let bias = |name: &str| -> Result<Array1<f32>, String> {
            let values = tensors
                .get(name)
                .ok_or_else(|| format!("DouZero 缺少 {name}"))?
                .expect_shape(&[512])?;
            Ok(Array1::from_iter(values.iter().copied()))
        };
        let bias = bias("lstm.bias_ih_l0")? + bias("lstm.bias_hh_l0")?;
        Ok(Self {
            input,
            recurrent,
            bias,
        })
    }

    pub(crate) fn encode_history(&self, history: &[f32]) -> Array1<f32> {
        let history = ArrayView2::from_shape((5, 162), history).expect("15 public moves");
        let inputs = history.dot(&self.input.t());
        let mut hidden = Array1::zeros(128);
        let mut cell = Array1::<f32>::zeros(128);
        for input in inputs.rows() {
            let gates = &input + &self.recurrent.dot(&hidden) + &self.bias;
            for index in 0..128 {
                let input_gate = sigmoid(gates[index]);
                let forget_gate = sigmoid(gates[index + 128]);
                let candidate = gates[index + 256].tanh();
                let output_gate = sigmoid(gates[index + 384]);
                cell[index] = forget_gate * cell[index] + input_gate * candidate;
                hidden[index] = output_gate * cell[index].tanh();
            }
        }
        hidden
    }
}

pub(super) fn build_head(
    tensors: &TensorMap,
    width: usize,
    device: DevicePreference,
) -> Result<Network, String> {
    tensors
        .get("dense6.weight")
        .ok_or("DouZero 缺少评分层")?
        .expect_shape(&[1, 512])?;
    let mut builder = NetworkBuilder::new(vec![width + 128]);
    let mut x = 0;
    for layer in 1..=6 {
        x = builder.linear(x, tensors, &format!("dense{layer}"), layer < 6)?;
    }
    builder.finish(x, device)
}

fn sigmoid(value: f32) -> f32 {
    1.0 / (1.0 + (-value).exp())
}
