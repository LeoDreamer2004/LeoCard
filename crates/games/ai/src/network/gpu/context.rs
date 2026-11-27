use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferUsages, ComputePipeline, ComputePipelineDescriptor, Device,
    DeviceDescriptor, DeviceType, Instance, PipelineLayout, PipelineLayoutDescriptor, Queue,
    RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, ShaderStages,
};

static CONTEXT: OnceLock<Result<Arc<GpuContext>, String>> = OnceLock::new();

pub(crate) struct GpuContext {
    pub device: Device,
    pub queue: Queue,
    pub bindings: BindGroupLayout,
    layout: PipelineLayout,
    pipelines: Mutex<BTreeMap<&'static str, Arc<ComputePipeline>>>,
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
        let entries: Vec<_> = (0..6)
            .map(|binding| BindGroupLayoutEntry {
                binding,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: if binding == 5 {
                        BufferBindingType::Uniform
                    } else {
                        BufferBindingType::Storage {
                            read_only: binding != 4,
                        }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let bindings = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("AI tensor bindings"),
            entries: &entries,
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("AI operation layout"),
            bind_group_layouts: &[Some(&bindings)],
            immediate_size: 0,
        });
        Ok(Arc::new(Self {
            device,
            queue,
            bindings,
            layout,
            pipelines: Mutex::new(BTreeMap::new()),
        }))
    }

    pub fn pipeline(&self, source: &'static str) -> Result<Arc<ComputePipeline>, String> {
        let mut pipelines = self.pipelines.lock().map_err(|e| e.to_string())?;
        if let Some(pipeline) = pipelines.get(source) {
            return Ok(pipeline.clone());
        }
        let source_text = format!("{}\n{}", include_str!("../operation/bindings.wgsl"), source);
        let shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("AI operation"),
            source: ShaderSource::Wgsl(Cow::Owned(source_text)),
        });
        let pipeline = Arc::new(
            self.device
                .create_compute_pipeline(&ComputePipelineDescriptor {
                    label: Some("AI inference"),
                    layout: Some(&self.layout),
                    module: &shader,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    cache: None,
                }),
        );
        pipelines.insert(source, pipeline.clone());
        Ok(pipeline)
    }

    pub fn buffer(&self, values: &[f32], usage: BufferUsages) -> Buffer {
        let bytes: Vec<_> = values.iter().flat_map(|n| n.to_le_bytes()).collect();
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("AI tensor"),
                contents: if bytes.is_empty() { &[0; 4] } else { &bytes },
                usage,
            })
    }
}
