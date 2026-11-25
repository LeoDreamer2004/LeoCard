use crate::{DevicePreference, ModelId, Network, TensorMap, model_spec};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

static MODELS: OnceLock<RwLock<BTreeMap<ModelId, Arc<Package>>>> = OnceLock::new();

struct Package {
    tensors: Vec<TensorMap>,
    networks: Mutex<BTreeMap<(String, DevicePreference), Arc<Network>>>,
}

/// A match keeps its own handle, so changing or deleting installed models cannot
/// replace the weights/device of robots already playing.
#[derive(Clone)]
pub struct LoadedModel {
    pub id: ModelId,
    pub device: DevicePreference,
    package: Arc<Package>,
}

impl fmt::Debug for LoadedModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadedModel")
            .field("id", &self.id)
            .field("device", &self.device)
            .finish_non_exhaustive()
    }
}

impl PartialEq for LoadedModel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.device == other.device
            && Arc::ptr_eq(&self.package, &other.package)
    }
}

impl LoadedModel {
    pub fn tensors(&self, index: usize) -> Result<&TensorMap, String> {
        self.package
            .tensors
            .get(index)
            .ok_or_else(|| "模型权重组缺失".into())
    }

    pub fn network(
        &self,
        key: &str,
        create: impl FnOnce() -> Result<Network, String>,
    ) -> Result<Arc<Network>, String> {
        let mut networks = self.package.networks.lock().map_err(|e| e.to_string())?;
        let key = (key.to_owned(), self.device);
        if let Some(network) = networks.get(&key) {
            return Ok(network.clone());
        }
        let network = Arc::new(create()?);
        networks.insert(key, network.clone());
        Ok(network)
    }
}

/// Only fully downloaded, converted and validated packages enter this store.
pub struct ModelRegistry;

impl ModelRegistry {
    pub fn install(id: ModelId, tensors: Vec<TensorMap>) -> Result<(), String> {
        if tensors.len() != model_spec(id).files.len() || tensors.iter().any(TensorMap::is_empty) {
            return Err("模型权重组不完整".into());
        }
        let package = Arc::new(Package {
            tensors,
            networks: Mutex::new(BTreeMap::new()),
        });
        MODELS
            .get_or_init(|| RwLock::new(BTreeMap::new()))
            .write()
            .map_err(|e| e.to_string())?
            .insert(id, package);
        Ok(())
    }

    pub fn get(id: ModelId, device: DevicePreference) -> Option<LoadedModel> {
        let package = MODELS.get()?.read().ok()?.get(&id)?.clone();
        Some(LoadedModel {
            id,
            device,
            package,
        })
    }

    pub fn remove(id: ModelId) {
        if let Some(models) = MODELS.get()
            && let Ok(mut models) = models.write()
        {
            models.remove(&id);
        }
    }
}
