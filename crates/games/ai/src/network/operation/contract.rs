use std::fmt::Debug;

pub(crate) struct CpuInput<'a> {
    pub values: &'a [f32],
    pub other: &'a [f32],
    pub shape: &'a [usize],
    pub batch: usize,
}

pub(crate) struct GpuKernel<'a> {
    pub source: &'static str,
    pub weight: &'a [f32],
    pub bias: &'a [f32],
    pub parameters: [u32; 8],
}

impl<'a> GpuKernel<'a> {
    pub fn new(
        source: &'static str,
        dimensions: [usize; 4],
        relu: bool,
        weight: &'a [f32],
        bias: &'a [f32],
    ) -> Self {
        let [inputs, outputs, height, width] = dimensions.map(|n| n as u32);
        Self {
            source,
            weight,
            bias,
            parameters: [0, inputs, outputs, height, width, 0, u32::from(relu), 0],
        }
    }
}

pub(crate) trait Operation: Debug + Send + Sync {
    fn cpu(&self, input: CpuInput<'_>) -> Result<Vec<f32>, String>;
    fn gpu(&self, input_shape: &[usize]) -> GpuKernel<'_>;

    fn other_input(&self) -> Option<usize> {
        None
    }

    fn validate_input(&self, _input: &[f32], _batch: usize) -> Result<(), String> {
        Ok(())
    }
}
