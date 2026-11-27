use crate::player::config_file;
use leocard_ai::{ModelId, ModelRegistry, TensorArchive, TensorMap, model_spec};
use leocard_blackjack::BlackjackTransformerBot;
use leocard_doudizhu::DouzeroBot;
use leocard_guandan::GuandanActorBot;
use leocard_mahjong::KdensBot;
use leocard_platform::storage as fs;
use leocard_texas_holdem::TexasPpoBot;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
struct Manifest {
    version: String,
    hashes: Vec<String>,
}

fn directory(id: ModelId) -> Result<PathBuf, String> {
    Ok(config_file("models")
        .ok_or("无法确定模型缓存目录")?
        .join(model_spec(id).key)
        .join(model_spec(id).version))
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|n| format!("{n:02x}"))
        .collect()
}

pub(super) fn validate(id: ModelId, tensors: &[TensorMap]) -> Result<(), String> {
    match id {
        ModelId::Douzero => DouzeroBot::validate(tensors),
        ModelId::Kdens3 => KdensBot::validate(tensors),
        ModelId::GuandanActor => GuandanActorBot::validate(tensors),
        ModelId::BlackjackTransformer => BlackjackTransformerBot::validate(tensors),
        ModelId::TexasPpo => TexasPpoBot::validate(tensors),
    }
}

pub(super) fn load(id: ModelId) -> Result<bool, String> {
    let directory = directory(id)?;
    let bytes = match fs::read(directory.join("manifest.json")) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if manifest.version != model_spec(id).version
        || manifest.hashes.len() != model_spec(id).files.len()
    {
        return Err("模型缓存版本错误，请重新下载".into());
    }
    let mut tensors = Vec::new();
    for (file, expected) in model_spec(id).files.iter().zip(manifest.hashes) {
        let bytes =
            fs::read(directory.join(format!("{}.lcai", file.name))).map_err(|e| e.to_string())?;
        if digest(&bytes) != expected {
            return Err("模型缓存损坏，请重新下载".into());
        }
        tensors.push(TensorArchive::decode(&bytes)?);
    }
    validate(id, &tensors)?;
    ModelRegistry::install(id, tensors)?;
    Ok(true)
}

pub(super) fn save(id: ModelId, files: &[Vec<u8>]) -> Result<(), String> {
    let directory = directory(id)?;
    let mut hashes = Vec::new();
    for (file, bytes) in model_spec(id).files.iter().zip(files) {
        fs::write_atomic(directory.join(format!("{}.lcai", file.name)), bytes)
            .map_err(|e| e.to_string())?;
        hashes.push(digest(bytes));
    }
    let manifest = Manifest {
        version: model_spec(id).version.to_owned(),
        hashes,
    };
    fs::write_atomic(
        directory.join("manifest.json"),
        serde_json::to_vec(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

pub(super) fn remove(id: ModelId) -> Result<(), String> {
    let directory = directory(id)?;
    for name in model_spec(id)
        .files
        .iter()
        .map(|f| format!("{}.lcai", f.name))
        .chain(["manifest.json".to_owned()])
    {
        if let Err(e) = fs::remove_file(directory.join(name))
            && e.kind() != ErrorKind::NotFound
        {
            return Err(e.to_string());
        }
    }
    Ok(())
}
