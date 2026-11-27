use super::{CpuInput, GpuKernel, Operation};

#[derive(Debug)]
pub(crate) struct Add {
    pub other: usize,
    pub relu: bool,
}

impl Operation for Add {
    fn cpu(&self, input: CpuInput<'_>) -> Result<Vec<f32>, String> {
        Ok(input
            .values
            .iter()
            .zip(input.other)
            .map(|(x, y)| if self.relu { (x + y).max(0.0) } else { x + y })
            .collect())
    }

    fn gpu(&self, _shape: &[usize]) -> GpuKernel<'_> {
        GpuKernel::new(include_str!("add.wgsl"), [0; 4], self.relu, &[], &[])
    }

    fn other_input(&self) -> Option<usize> {
        Some(self.other)
    }
}
