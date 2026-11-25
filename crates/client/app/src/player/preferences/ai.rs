use leocard_ai::{DevicePreference, ModelId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct AiPreferences {
    pub device: DevicePreference,
    pub download_proxy_port: String,
    pub enabled: BTreeSet<ModelId>,
}
