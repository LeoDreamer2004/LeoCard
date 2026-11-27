use super::{CpuInput, GpuKernel, Operation};
use crate::Tensor;

#[derive(Debug)]
pub(crate) struct LayerNorm {
    pub weight: Tensor,
    pub bias: Tensor,
    pub relu: bool,
}

impl Operation for LayerNorm {
    fn cpu(&self, input: CpuInput<'_>) -> Result<Vec<f32>, String> {
        let width = self.weight.values.len();
        Ok(input
            .values
            .chunks_exact(width)
            .flat_map(|row| {
                let mean = row.iter().sum::<f32>() / width as f32;
                let variance = row.iter().map(|n| (n - mean).powi(2)).sum::<f32>() / width as f32;
                let scale = (variance + 1e-5).sqrt().recip();
                row.iter().enumerate().map(move |(i, n)| {
                    let value = (n - mean) * scale * self.weight.values[i] + self.bias.values[i];
                    if self.relu { value.max(0.0) } else { value }
                })
            })
            .collect())
    }

    fn gpu(&self, _shape: &[usize]) -> GpuKernel<'_> {
        GpuKernel::new(
            include_str!("layer_norm.wgsl"),
            [0, self.weight.shape[0], 0, 0],
            self.relu,
            &self.weight.values,
            &self.bias.values,
        )
    }
}
