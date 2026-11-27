use super::{CpuInput, GpuKernel, Operation};
use crate::Tensor;

#[derive(Debug)]
pub(crate) struct Embedding {
    pub weight: Tensor,
    pub positions: Tensor,
}

impl Operation for Embedding {
    fn cpu(&self, input: CpuInput<'_>) -> Result<Vec<f32>, String> {
        let width = self.weight.shape[1];
        Ok(input
            .values
            .iter()
            .enumerate()
            .flat_map(|(row, token)| {
                let position = self.positions.shape[0] - input.batch + row;
                (0..width).map(move |i| {
                    self.weight.values[*token as usize * width + i]
                        + self.positions.values[position * width + i]
                })
            })
            .collect())
    }

    fn gpu(&self, _shape: &[usize]) -> GpuKernel<'_> {
        GpuKernel::new(
            include_str!("embedding.wgsl"),
            [0, self.weight.shape[1], self.positions.shape[0], 0],
            false,
            &self.weight.values,
            &self.positions.values,
        )
    }

    fn validate_input(&self, input: &[f32], batch: usize) -> Result<(), String> {
        if batch > self.positions.shape[0]
            || input
                .iter()
                .any(|n| *n < 0.0 || n.fract() != 0.0 || *n >= self.weight.shape[0] as f32)
        {
            return Err("模型词元或序列长度错误".into());
        }
        Ok(())
    }
}
