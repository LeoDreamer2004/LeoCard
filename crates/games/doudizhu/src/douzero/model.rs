//! CPU implementation of DouZero's published LSTM/MLP architecture.
//! Weights are stored as f16 and expanded to f32 once per used role.

use half::f16;
use ndarray::{Array1, Array2, ArrayView2};
use std::sync::OnceLock;

static MODELS: [OnceLock<Model>; 3] = [const { OnceLock::new() }; 3];

pub(crate) struct Model {
    input: Array2<f32>,
    recurrent: Array2<f32>,
    bias: Array1<f32>,
    dense: Vec<Linear>,
}

struct Linear {
    weight: Array2<f32>,
    bias: Array1<f32>,
}

impl Model {
    pub(crate) fn for_role(role: usize) -> &'static Self {
        MODELS[role].get_or_init(|| {
            let bytes: &[u8] = match role {
                0 => include_bytes!("../../models/douzero/landlord.f16"),
                1 => include_bytes!("../../models/douzero/landlord_down.f16"),
                _ => include_bytes!("../../models/douzero/landlord_up.f16"),
            };
            Self::read(bytes, if role == 0 { 373 } else { 484 })
        })
    }

    fn read(bytes: &[u8], width: usize) -> Self {
        assert_eq!(&bytes[..8], b"DZERO01\0");
        let mut values = bytes[8..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|bytes| f16::from_le_bytes([bytes[0], bytes[1]]).to_f32());
        let mut matrix = |rows, columns| {
            Array2::from_shape_vec(
                (rows, columns),
                values.by_ref().take(rows * columns).collect(),
            )
            .expect("bundled matrix dimensions")
        };
        let input = matrix(512, 162);
        let recurrent = matrix(512, 128);
        let bias = Array1::from_iter(values.by_ref().take(512))
            + Array1::from_iter(values.by_ref().take(512));
        let mut dense = Vec::new();
        for (inputs, outputs) in [
            (width + 128, 512),
            (512, 512),
            (512, 512),
            (512, 512),
            (512, 512),
            (512, 1),
        ] {
            let weight = Array2::from_shape_vec(
                (outputs, inputs),
                values.by_ref().take(outputs * inputs).collect(),
            )
            .expect("bundled dense dimensions");
            let bias = Array1::from_iter(values.by_ref().take(outputs));
            assert_eq!(bias.len(), outputs);
            dense.push(Linear { weight, bias });
        }
        assert!(values.next().is_none(), "unexpected bundled weights");
        Self {
            input,
            recurrent,
            bias,
            dense,
        }
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

    pub(crate) fn scores(&self, hidden: &Array1<f32>, features: &[Vec<f32>]) -> Vec<f32> {
        let mut batch = Array2::from_shape_fn(
            (features.len(), hidden.len() + features[0].len()),
            |(row, column)| {
                if column < hidden.len() {
                    hidden[column]
                } else {
                    features[row][column - hidden.len()]
                }
            },
        );
        for (index, layer) in self.dense.iter().enumerate() {
            batch = batch.dot(&layer.weight.t()) + &layer.bias;
            if index + 1 < self.dense.len() {
                batch.mapv_inplace(|value| value.max(0.0));
            }
        }
        batch.column(0).to_vec()
    }
}

fn sigmoid(value: f32) -> f32 {
    1.0 / (1.0 + (-value).exp())
}
