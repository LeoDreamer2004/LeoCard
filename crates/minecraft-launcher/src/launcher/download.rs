use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub(super) struct Downloader {
    client: Client,
}

#[derive(Clone)]
pub(super) struct DownloadFile {
    pub url: String,
    pub target: PathBuf,
    pub sha1: Option<String>,
}

impl Downloader {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .user_agent(format!(
                "LeoCard/{} (Minecraft launcher)",
                env!("CARGO_PKG_VERSION")
            ))
            .timeout(Duration::from_secs(45))
            .build()
            .map_err(|error| format!("无法创建下载客户端：{error}"))?;
        Ok(Self { client })
    }

    pub fn json<T: DeserializeOwned>(&self, url: &str) -> Result<T, String> {
        self.check_url(url)?;
        self.client
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.json())
            .map_err(|error| format!("获取 Minecraft 数据失败：{error}"))
    }

    pub fn text(&self, url: &str) -> Result<String, String> {
        self.check_url(url)?;
        self.client
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.text())
            .map_err(|error| format!("获取版本信息失败：{error}"))
    }

    pub fn json_with_key<T: DeserializeOwned>(&self, url: &str, key: &str) -> Result<T, String> {
        self.check_url(url)?;
        self.client
            .get(url)
            .header("x-api-key", key)
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.json())
            .map_err(|error| format!("CurseForge 请求失败：{error}"))
    }

    pub fn json_query<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, &str)],
    ) -> Result<T, String> {
        self.check_url(url)?;
        self.client
            .get(url)
            .query(query)
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.json())
            .map_err(|error| format!("搜索 Minecraft 资源失败：{error}"))
    }

    pub fn file(&self, file: &DownloadFile) -> Result<(), String> {
        if file.target.is_file() && self.valid_hash(&file.target, file.sha1.as_deref())? {
            return Ok(());
        }
        self.check_url(&file.url)?;
        let parent = file.target.parent().ok_or("下载文件没有父目录")?;
        fs::create_dir_all(parent).map_err(|error| format!("无法创建下载目录：{error}"))?;
        let temporary = file.target.with_extension("part");
        let result = (|| {
            let mut response = self
                .client
                .get(&file.url)
                .send()
                .and_then(|response| response.error_for_status())
                .map_err(|error| format!("下载 {} 失败：{error}", file.url))?;
            let mut output = File::create(&temporary)
                .map_err(|error| format!("无法写入 {}：{error}", temporary.display()))?;
            let mut hasher = Sha1::new();
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let count = response
                    .read(&mut buffer)
                    .map_err(|error| error.to_string())?;
                if count == 0 {
                    break;
                }
                output
                    .write_all(&buffer[..count])
                    .map_err(|error| error.to_string())?;
                hasher.update(&buffer[..count]);
            }
            output.flush().map_err(|error| error.to_string())?;
            if let Some(expected) = &file.sha1
                && format!("{:x}", hasher.finalize()) != expected.to_ascii_lowercase()
            {
                return Err(format!("下载校验失败：{}", file.target.display()));
            }
            if file.target.exists() {
                fs::remove_file(&file.target).map_err(|error| error.to_string())?;
            }
            fs::rename(&temporary, &file.target).map_err(|error| error.to_string())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    pub fn files(
        &self,
        files: Vec<DownloadFile>,
        mut progress: impl FnMut(usize, usize),
    ) -> Result<(), String> {
        let total = files.len();
        if total == 0 {
            return Ok(());
        }
        let files = Arc::new(files);
        let next = AtomicUsize::new(0);
        let (sender, receiver) = mpsc::channel();
        let mut first_error = None;
        thread::scope(|scope| {
            for _ in 0..total.min(8) {
                let files = Arc::clone(&files);
                let sender = sender.clone();
                let downloader = self.clone();
                let next = &next;
                scope.spawn(move || {
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(file) = files.get(index) else {
                            break;
                        };
                        if sender.send(downloader.file(file)).is_err() {
                            break;
                        }
                    }
                });
            }
            drop(sender);
            for (index, result) in receiver.into_iter().enumerate() {
                if let Err(error) = result {
                    first_error.get_or_insert(error);
                }
                if index == 0 || index + 1 == total || (index + 1) % 25 == 0 {
                    progress(index + 1, total);
                }
            }
        });
        first_error.map_or(Ok(()), Err)
    }

    fn valid_hash(&self, path: &Path, expected: Option<&str>) -> Result<bool, String> {
        let Some(expected) = expected else {
            return Ok(true);
        };
        let mut file = File::open(path).map_err(|error| error.to_string())?;
        let mut hasher = Sha1::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        Ok(format!("{:x}", hasher.finalize()) == expected.to_ascii_lowercase())
    }

    fn check_url(&self, url: &str) -> Result<(), String> {
        if url.starts_with("https://") {
            Ok(())
        } else {
            Err(format!("仅允许 HTTPS 下载：{url}"))
        }
    }
}
