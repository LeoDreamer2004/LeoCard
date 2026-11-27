use super::cpu;
use super::gpu::{GpuContext, GpuNetwork};
use super::operation::Operation;
use crate::{DevicePreference, InferenceDevice};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub(super) struct Step {
    pub input: usize,
    pub shape: Vec<usize>,
    pub operation: Arc<dyn Operation>,
}

pub struct Network {
    pub(super) input_shape: Vec<usize>,
    pub(super) steps: Vec<Step>,
    pub(super) output: usize,
    preference: DevicePreference,
    gpu: Mutex<Option<Result<GpuNetwork, String>>>,
}

impl Network {
    pub(super) fn new(
        input_shape: Vec<usize>,
        steps: Vec<Step>,
        output: usize,
        preference: DevicePreference,
    ) -> Self {
        Self {
            input_shape,
            steps,
            output,
            preference,
            gpu: Mutex::new(None),
        }
    }

    pub fn run(&self, input: &[f32], batch: usize) -> Result<Vec<f32>, String> {
        let expected = self
            .input_shape
            .iter()
            .product::<usize>()
            .checked_mul(batch);
        if batch == 0 || expected != Some(input.len()) || input.iter().any(|n| !n.is_finite()) {
            return Err("模型输入尺寸错误或包含非有限数值".into());
        }
        for step in &self.steps {
            step.operation.validate_input(input, batch)?;
        }
        if self.preference != DevicePreference::Cpu {
            let mut gpu = self.gpu.lock().map_err(|e| e.to_string())?;
            let state = gpu.get_or_insert_with(|| GpuNetwork::new(self));
            let result = match state {
                Ok(gpu) => gpu.run(self, input, batch),
                Err(error) => Err(error.clone()),
            };
            match result {
                Ok(output) => return Ok(output),
                Err(error) if self.preference == DevicePreference::Gpu => return Err(error),
                Err(error) => *state = Err(error),
            }
        }
        cpu::run(self, input, batch)
    }

    pub fn available_device(preference: DevicePreference) -> Result<InferenceDevice, String> {
        if preference == DevicePreference::Cpu {
            return Ok(InferenceDevice::Cpu);
        }
        match GpuContext::shared() {
            Ok(_) => return Ok(InferenceDevice::Gpu),
            Err(error) if preference == DevicePreference::Gpu => return Err(error),
            Err(_) => {}
        }
        Ok(InferenceDevice::Cpu)
    }

    pub(super) fn shape(&self, slot: usize) -> &[usize] {
        if slot == 0 {
            &self.input_shape
        } else {
            &self.steps[slot - 1].shape
        }
    }
}
