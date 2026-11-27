use super::{CpuInput, GpuKernel, Operation};
use std::f32::consts;

#[derive(Debug)]
pub(crate) struct Gelu;

impl Operation for Gelu {
    fn cpu(&self, input: CpuInput<'_>) -> Result<Vec<f32>, String> {
        Ok(input
            .values
            .iter()
            .copied()
            .map(|x| {
                // Abramowitz-Stegun erf approximation, maximum error below 1.5e-7.
                let z = x * consts::FRAC_1_SQRT_2;
                let t = 1.0 / (1.0 + 0.327_591_1 * z.abs());
                let polynomial = (((((1.061_405_4 * t - 1.453_152_1) * t) + 1.421_413_8) * t
                    - 0.284_496_72)
                    * t
                    + 0.254_829_6)
                    * t;
                0.5 * x * (1.0 + z.signum() * (1.0 - polynomial * (-z * z).exp()))
            })
            .collect())
    }

    fn gpu(&self, _shape: &[usize]) -> GpuKernel<'_> {
        GpuKernel::new(include_str!("gelu.wgsl"), [0; 4], false, &[], &[])
    }
}
