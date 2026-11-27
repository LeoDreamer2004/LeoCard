use super::{CpuInput, GpuKernel, Operation};
use crate::Tensor;
use ndarray::ArrayView2;

#[derive(Debug)]
pub(crate) struct Linear {
    pub weight: Tensor,
    pub bias: Tensor,
    pub relu: bool,
}

impl Operation for Linear {
    fn cpu(&self, input: CpuInput<'_>) -> Result<Vec<f32>, String> {
        let outputs = self.weight.shape[0];
        let inputs = self.weight.shape[1];
        let x = ArrayView2::from_shape((input.batch, inputs), input.values)
            .map_err(|e| e.to_string())?;
        let w = ArrayView2::from_shape((outputs, inputs), self.weight.values.as_ref())
            .map_err(|e| e.to_string())?;
        let mut y = x.dot(&w.t());
        for mut row in y.rows_mut() {
            for (value, bias) in row.iter_mut().zip(self.bias.values.iter()) {
                *value += bias;
                if self.relu {
                    *value = value.max(0.0);
                }
            }
        }
        Ok(y.into_raw_vec_and_offset().0)
    }

    fn gpu(&self, _input_shape: &[usize]) -> GpuKernel<'_> {
        GpuKernel::new(
            include_str!("linear.wgsl"),
            [self.weight.shape[1], self.weight.shape[0], 0, 0],
            self.relu,
            &self.weight.values,
            &self.bias.values,
        )
    }
}
