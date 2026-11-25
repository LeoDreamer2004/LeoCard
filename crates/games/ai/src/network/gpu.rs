use super::state::{Network, Operation};
use std::borrow::Cow;
use std::sync::{Arc, OnceLock, mpsc};
use std::time::Duration;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor,
    Device, DeviceDescriptor, DeviceType, Instance, MapMode, PollType, Queue,
    RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
};

static CONTEXT: OnceLock<Result<Arc<GpuContext>, String>> = OnceLock::new();

pub(super) struct GpuContext {
    device: Device,
    queue: Queue,
    pipeline: ComputePipeline,
}

impl GpuContext {
    pub fn shared() -> Result<Arc<Self>, String> {
        CONTEXT.get_or_init(Self::create).clone()
    }

    fn create() -> Result<Arc<Self>, String> {
        let instance = Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
                .map_err(|e| format!("无法初始化 AI GPU：{e}"))?;
        if adapter.get_info().device_type == DeviceType::Cpu {
            return Err("未检测到支持计算的硬件 GPU".into());
        }
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            label: Some("LeoCard AI"),
            ..Default::default()
        }))
        .map_err(|e| format!("无法创建 AI GPU 设备：{e}"))?;
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("LeoCard AI kernels"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("kernels.wgsl"))),
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("LeoCard AI inference"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(Arc::new(Self {
            device,
            queue,
            pipeline,
        }))
    }

    fn buffer(&self, values: &[f32], usage: BufferUsages) -> Buffer {
        let bytes: Vec<_> = values.iter().flat_map(|n| n.to_le_bytes()).collect();
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("AI tensor"),
                contents: if bytes.is_empty() { &[0; 4] } else { &bytes },
                usage,
            })
    }
}

pub(super) struct GpuNetwork {
    context: Arc<GpuContext>,
    weights: Vec<(Buffer, Buffer)>,
}

impl GpuNetwork {
    pub fn new(network: &Network) -> Result<Self, String> {
        let context = GpuContext::shared()?;
        let weights = network
            .steps
            .iter()
            .map(|step| {
                let (weight, bias): (&[f32], &[f32]) = match &step.operation {
                    Operation::Linear { weight, bias, .. }
                    | Operation::Conv3x3 { weight, bias, .. } => {
                        (weight.values.as_ref(), bias.values.as_ref())
                    }
                    Operation::AddRelu { .. } => (&[], &[]),
                };
                (
                    context.buffer(weight, BufferUsages::STORAGE),
                    context.buffer(bias, BufferUsages::STORAGE),
                )
            })
            .collect();
        Ok(Self { context, weights })
    }

    pub fn run(&self, network: &Network, input: &[f32], batch: usize) -> Result<Vec<f32>, String> {
        let context = &self.context;
        let mut slots = vec![context.buffer(input, BufferUsages::STORAGE)];
        let mut encoder = context
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        let layout = context.pipeline.get_bind_group_layout(0);
        for (index, step) in network.steps.iter().enumerate() {
            let length = step.shape.iter().product::<usize>() * batch;
            let output = context.device.create_buffer(&BufferDescriptor {
                label: Some("AI activation"),
                size: (length * 4) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let mut parameters = [0u32; 8];
            parameters[5] = batch as u32;
            parameters[7] = length as u32;
            let other = match &step.operation {
                Operation::Linear { weight, relu, .. } => {
                    parameters[0] = 0;
                    parameters[1] = weight.shape[1] as u32;
                    parameters[2] = weight.shape[0] as u32;
                    parameters[6] = u32::from(*relu);
                    step.input
                }
                Operation::Conv3x3 { weight, relu, .. } => {
                    let shape = network.shape(step.input);
                    parameters[0] = 1;
                    parameters[1] = shape[0] as u32;
                    parameters[2] = weight.shape[0] as u32;
                    parameters[3] = shape[1] as u32;
                    parameters[4] = shape[2] as u32;
                    parameters[6] = u32::from(*relu);
                    step.input
                }
                Operation::AddRelu { other } => {
                    parameters[0] = 2;
                    parameters[6] = 1;
                    *other
                }
            };
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
                &self.weights[index].0,
                &self.weights[index].1,
                &slots[other],
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
                layout: &layout,
                entries: &entries,
            });
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
                pass.set_pipeline(&context.pipeline);
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
