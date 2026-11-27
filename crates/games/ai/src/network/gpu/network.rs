use super::GpuContext;
use crate::network::state::Network;
use std::sync::{Arc, mpsc};
use std::time::Duration;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, MapMode, PollType,
};

struct GpuStep {
    pipeline: Arc<ComputePipeline>,
    weight: Buffer,
    bias: Buffer,
    parameters: [u32; 8],
    other: Option<usize>,
}

pub(crate) struct GpuNetwork {
    context: Arc<GpuContext>,
    steps: Vec<GpuStep>,
}

impl GpuNetwork {
    pub fn new(network: &Network) -> Result<Self, String> {
        let context = GpuContext::shared()?;
        let steps = network
            .steps
            .iter()
            .map(|step| {
                let kernel = step.operation.gpu(network.shape(step.input));
                Ok(GpuStep {
                    pipeline: context.pipeline(kernel.source)?,
                    weight: context.buffer(kernel.weight, BufferUsages::STORAGE),
                    bias: context.buffer(kernel.bias, BufferUsages::STORAGE),
                    parameters: kernel.parameters,
                    other: step.operation.other_input(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self { context, steps })
    }

    pub fn run(&self, network: &Network, input: &[f32], batch: usize) -> Result<Vec<f32>, String> {
        let context = &self.context;
        let mut slots = vec![context.buffer(input, BufferUsages::STORAGE)];
        let mut encoder = context
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        for (step, gpu) in network.steps.iter().zip(&self.steps) {
            let length = step.shape.iter().product::<usize>() * batch;
            let output = context.device.create_buffer(&BufferDescriptor {
                label: Some("AI activation"),
                size: (length * 4) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let mut parameters = gpu.parameters;
            parameters[5] = batch as u32;
            parameters[7] = length as u32;
            let bytes: Vec<_> = parameters.iter().flat_map(|n| n.to_le_bytes()).collect();
            let parameters = context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("AI operation"),
                    contents: &bytes,
                    usage: BufferUsages::UNIFORM,
                });
            let buffers = [
                &slots[step.input],
                &gpu.weight,
                &gpu.bias,
                &slots[gpu.other.unwrap_or(step.input)],
                &output,
                &parameters,
            ];
            let entries: Vec<_> = buffers
                .iter()
                .enumerate()
                .map(|(binding, buffer)| BindGroupEntry {
                    binding: binding as u32,
                    resource: buffer.as_entire_binding(),
                })
                .collect();
            let group = context.device.create_bind_group(&BindGroupDescriptor {
                label: Some("AI operation"),
                layout: &context.bindings,
                entries: &entries,
            });
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
                pass.set_pipeline(&gpu.pipeline);
                pass.set_bind_group(0, &group, &[]);
                pass.dispatch_workgroups(length.div_ceil(64) as u32, 1, 1);
            }
            slots.push(output);
        }
        let length = network.shape(network.output).iter().product::<usize>() * batch;
        let readback = context.device.create_buffer(&BufferDescriptor {
            label: Some("AI result"),
            size: (length * 4) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&slots[network.output], 0, &readback, 0, (length * 4) as u64);
        let submission = context.queue.submit([encoder.finish()]);
        let (sender, receiver) = mpsc::channel();
        readback.slice(..).map_async(MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        context
            .device
            .poll(PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(30)),
            })
            .map_err(|e| format!("GPU 推理等待失败：{e}"))?;
        receiver
            .recv_timeout(Duration::from_secs(1))
            .map_err(|e| e.to_string())?
            .map_err(|e| format!("GPU 结果读取失败：{e}"))?;
        let output = {
            let mapped = readback
                .slice(..)
                .get_mapped_range()
                .map_err(|e| e.to_string())?;
            mapped
                .as_chunks::<4>()
                .0
                .iter()
                .map(|v| f32::from_le_bytes(*v))
                .collect::<Vec<_>>()
        };
        readback.unmap();
        if output.iter().any(|n| !n.is_finite()) {
            return Err("GPU 模型输出包含非有限数值".into());
        }
        Ok(output)
    }
}
