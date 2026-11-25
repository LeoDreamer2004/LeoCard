use super::{download, storage};
use bevy::prelude::*;
use leocard_ai::{DevicePreference, InferenceDevice, ModelId, ModelRegistry, Network, model_spec};
use std::collections::BTreeMap;
use std::sync::{Mutex, mpsc};
use std::thread;

#[derive(Clone, Debug)]
pub(crate) enum ModelStatus {
    Missing,
    Loading,
    Downloading { completed: u64, total: u64 },
    Converting,
    Ready,
    Failed(String),
}

impl ModelStatus {
    pub fn busy(&self) -> bool {
        matches!(
            self,
            Self::Loading | Self::Downloading { .. } | Self::Converting
        )
    }

    pub fn ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

pub(super) enum ModelEvent {
    Status(ModelId, ModelStatus),
    Device(Result<InferenceDevice, String>),
}

#[derive(Resource)]
pub(crate) struct AiModels {
    pub states: BTreeMap<ModelId, ModelStatus>,
    pub gpu: Option<Result<InferenceDevice, String>>,
    pub(super) probing_gpu: bool,
    pub(super) sender: mpsc::Sender<ModelEvent>,
    pub(super) receiver: Mutex<mpsc::Receiver<ModelEvent>>,
}

impl Default for AiModels {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            states: ModelId::ALL
                .into_iter()
                .map(|id| (id, ModelStatus::Missing))
                .collect(),
            gpu: None,
            probing_gpu: false,
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

impl AiModels {
    pub fn probe_device(&mut self) {
        if self.gpu.is_none() && !self.probing_gpu {
            self.probing_gpu = true;
            let sender = self.sender.clone();
            thread::spawn(move || {
                let _ = sender.send(ModelEvent::Device(Network::available_device(
                    DevicePreference::Gpu,
                )));
            });
        }
    }

    pub fn status(&self, id: ModelId) -> &ModelStatus {
        &self.states[&id]
    }

    pub fn gpu_available(&self) -> bool {
        self.gpu.as_ref().is_some_and(Result::is_ok)
    }

    pub fn download(&mut self, id: ModelId, proxy_port: &str) {
        if self.status(id).busy() {
            return;
        }
        self.states.insert(
            id,
            ModelStatus::Downloading {
                completed: 0,
                total: model_spec(id).files.iter().map(|f| f.bytes).sum(),
            },
        );
        download::start(id, proxy_port.to_owned(), self.sender.clone());
    }

    pub fn remove(&mut self, id: ModelId) -> Result<(), String> {
        if self.status(id).busy() {
            return Err("请等待模型任务完成".into());
        }
        storage::remove(id)?;
        ModelRegistry::remove(id);
        self.states.insert(id, ModelStatus::Missing);
        Ok(())
    }
}
