use super::{download, storage};
use leocard_ai::{DevicePreference, ModelId, ModelRegistry};
use std::env;
use std::path::PathBuf;
use std::sync::mpsc;

#[test]
#[ignore = "downloads external models; requires an isolated LEOCARD_CONFIG_DIR"]
fn downloaded_models_convert_save_and_reload() {
    let config = PathBuf::from(env::var_os("LEOCARD_CONFIG_DIR").expect("isolated test config"));
    assert!(
        config.to_string_lossy().contains("ai-validation"),
        "use a dedicated ai-validation directory"
    );
    let (sender, _receiver) = mpsc::channel();
    for id in ModelId::ALL {
        download::download(id, "", &sender).unwrap();
        ModelRegistry::remove(id);
        assert!(storage::load(id).unwrap());
        assert!(ModelRegistry::get(id, DevicePreference::Cpu).is_some());
        storage::remove(id).unwrap();
        ModelRegistry::remove(id);
        assert!(!storage::load(id).unwrap());
    }
}
