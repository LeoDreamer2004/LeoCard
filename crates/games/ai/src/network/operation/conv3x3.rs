use super::{CpuInput, GpuKernel, Operation};
use crate::Tensor;
use ndarray::{Array2, ArrayView2};

#[derive(Debug)]
pub(crate) struct Conv3x3 {
    pub weight: Tensor,
    pub bias: Tensor,
    pub relu: bool,
}

impl Operation for Conv3x3 {
    fn cpu(&self, input: CpuInput<'_>) -> Result<Vec<f32>, String> {
        let [channels, height, width] = input.shape else {
            return Err("卷积输入尺寸错误".into());
        };
        let outputs = self.weight.shape[0];
        let area = height * width;
        let kernel = channels * 9;
        let w = ArrayView2::from_shape((outputs, kernel), self.weight.values.as_ref())
            .map_err(|e| e.to_string())?;
        let mut output = vec![0.0; input.batch * outputs * area];
        for n in 0..input.batch {
            let cols = Array2::from_shape_fn((area, kernel), |(position, k)| {
                let x = position % width;
                let y = position / width;
                let dx = k % 3;
                let dy = k / 3 % 3;
                let c = k / 9;
                if x + dx == 0 || y + dy == 0 || x + dx > *width || y + dy > *height {
                    0.0
                } else {
                    input.values[n * channels * area + c * area + (y + dy - 1) * width + x + dx - 1]
                }
            });
            let result = cols.dot(&w.t());
            for c in 0..outputs {
                for p in 0..area {
                    let value = result[(p, c)] + self.bias.values[c];
                    output[n * outputs * area + c * area + p] =
                        if self.relu { value.max(0.0) } else { value };
                }
            }
        }
        Ok(output)
    }

    fn gpu(&self, shape: &[usize]) -> GpuKernel<'_> {
        GpuKernel::new(
            include_str!("conv3x3.wgsl"),
            [shape[0], self.weight.shape[0], shape[1], shape[2]],
            self.relu,
            &self.weight.values,
            &self.bias.values,
        )
    }
}
