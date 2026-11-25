use super::state::{ModelEvent, ModelStatus};
use super::storage;
use leocard_ai::{ModelId, ModelRegistry, model_spec};
use reqwest::Proxy;
use reqwest::blocking::Client;
use std::io::Read;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

pub(super) fn start(id: ModelId, proxy_port: String, sender: Sender<ModelEvent>) {
    thread::spawn(move || {
        let result = download(id, &proxy_port, &sender);
        let status = match result {
            Ok(()) => ModelStatus::Ready,
            Err(error) => ModelStatus::Failed(error),
        };
        let _ = sender.send(ModelEvent::Status(id, status));
    });
}

pub(super) fn download(
    id: ModelId,
    proxy_port: &str,
    sender: &Sender<ModelEvent>,
) -> Result<(), String> {
    let mut client = Client::builder()
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(300))
        .user_agent("LeoCard model downloader");
    if !proxy_port.trim().is_empty() {
        let port = proxy_port
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|port| *port != 0)
            .ok_or("下载代理端口须为 1 到 65535")?;
        let proxy = Proxy::all(format!("http://127.0.0.1:{port}")).map_err(|e| e.to_string())?;
        client = client.proxy(proxy);
    }
    let client = client.build().map_err(|e| e.to_string())?;
    let total = model_spec(id).files.iter().map(|f| f.bytes).sum();
    let mut completed = 0;
    let mut files = Vec::new();
    let mut tensors = Vec::new();
    for file in model_spec(id).files {
        let mut response = client
            .get(file.url)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|e| format!("{} 下载失败：{e:#}", file.name))?;
        let mut bytes = Vec::with_capacity(file.bytes as usize);
        let mut buffer = [0u8; 65536];
        let mut last_progress = Instant::now();
        loop {
            let count = response
                .read(&mut buffer)
                .map_err(|e| format!("模型下载中断：{e}"))?;
            if count == 0 {
                break;
            }
            if bytes.len() as u64 + count as u64 > file.bytes {
                return Err("模型下载大小异常".into());
            }
            bytes.extend_from_slice(&buffer[..count]);
            if last_progress.elapsed() >= Duration::from_millis(150) {
                let _ = sender.send(ModelEvent::Status(
                    id,
                    ModelStatus::Downloading {
                        completed: completed + bytes.len() as u64,
                        total,
                    },
                ));
                last_progress = Instant::now();
            }
        }
        completed += file.bytes;
        let _ = sender.send(ModelEvent::Status(id, ModelStatus::Converting));
        let (encoded, converted) = file.convert(&bytes)?;
        files.push(encoded);
        tensors.push(converted);
    }
    storage::validate(id, &tensors)?;
    storage::save(id, &files)?;
    ModelRegistry::install(id, tensors)
}
