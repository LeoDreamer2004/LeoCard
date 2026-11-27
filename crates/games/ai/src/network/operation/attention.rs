use super::{CpuInput, GpuKernel, Operation};

#[derive(Debug)]
pub(crate) struct CausalAttention {
    pub heads: usize,
}

impl Operation for CausalAttention {
    fn cpu(&self, input: CpuInput<'_>) -> Result<Vec<f32>, String> {
        let width = input.shape[0] / 3;
        let head_width = width / self.heads;
        let scale = (head_width as f32).sqrt().recip();
        let mut output = vec![0.0; input.batch * width];
        for row in 0..input.batch {
            for head in 0..self.heads {
                let start = head * head_width;
                let query = &input.values[row * 3 * width + start..][..head_width];
                let scores: Vec<_> = (0..=row)
                    .map(|key| {
                        let key = &input.values[key * 3 * width + width + start..][..head_width];
                        query.iter().zip(key).map(|(q, k)| q * k).sum::<f32>() * scale
                    })
                    .collect();
                let maximum = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                let weights: Vec<_> = scores.iter().map(|score| (score - maximum).exp()).collect();
                let total = weights.iter().sum::<f32>();
                for (key, weight) in weights.iter().enumerate() {
                    for column in 0..head_width {
                        output[row * width + start + column] += weight / total
                            * input.values[key * 3 * width + 2 * width + start + column];
                    }
                }
            }
        }
        Ok(output)
    }

    fn gpu(&self, shape: &[usize]) -> GpuKernel<'_> {
        GpuKernel::new(
            include_str!("attention.wgsl"),
            [0, shape[0] / 3, self.heads, 0],
            false,
            &[],
            &[],
        )
    }
}
