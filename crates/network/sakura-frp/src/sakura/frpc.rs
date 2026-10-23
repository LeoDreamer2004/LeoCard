use super::api::SakuraClient;
use super::paths::data_root;
use md5::{Digest, Md5};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrpcProgress {
    ResolvingDownload,
    CheckingCache,
    Downloading { downloaded: u64, total: Option<u64> },
    Verifying,
    FetchingConfig,
    Starting,
    Started,
}

#[derive(Deserialize)]
struct ClientDownloads {
    frpc: ClientRelease,
}

#[derive(Deserialize)]
struct ClientRelease {
    ver: String,
    archs: HashMap<String, ClientArtifact>,
}

#[derive(Deserialize)]
struct ClientArtifact {
    url: String,
    hash: Option<String>,
}

pub struct RunningTunnel {
    pub id: u64,
    child: Child,
    config_path: PathBuf,
    pub log_path: PathBuf,
}

impl RunningTunnel {
    pub fn running(&mut self) -> Result<bool, String> {
        self.child
            .try_wait()
            .map(|status| status.is_none())
            .map_err(|error| format!("检查 frpc 状态失败：{error}"))
    }
}

impl Drop for RunningTunnel {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_file(&self.config_path);
    }
}

impl SakuraClient {
    pub fn start_tunnel(
        &self,
        id: u64,
        mut progress: impl FnMut(FrpcProgress),
    ) -> Result<RunningTunnel, String> {
        progress(FrpcProgress::ResolvingDownload);
        let downloads: ClientDownloads = self.public_json("/system/clients?download=true")?;
        let release = downloads.frpc;
        let artifact = release
            .archs
            .get(platform_arch()?)
            .ok_or("官方未提供当前系统架构的 frpc")?;
        let url = reqwest::Url::parse(&artifact.url)
            .map_err(|error| format!("frpc 下载地址无效：{error}"))?;
        if url.scheme() != "https" || url.host_str() != Some("nya.globalslb.net") {
            return Err("frpc 下载地址不是官方 HTTPS 地址".to_owned());
        }
        let version = url
            .path_segments()
            .and_then(|mut parts| {
                parts.position(|part| part == "frpc")?;
                parts.next().map(str::to_owned)
            })
            .unwrap_or_else(|| release.ver.clone());
        if version.is_empty()
            || !version
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        {
            return Err("frpc 版本名称无效".to_owned());
        }
        let expected = artifact
            .hash
            .as_deref()
            .ok_or("官方 frpc 下载信息缺少校验值")?
            .to_ascii_lowercase();
        if expected.len() != 32 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("官方 frpc 校验值无效".to_owned());
        }
        let root = data_root()?;
        let binary = root
            .join("bin")
            .join(&version)
            .join(if cfg!(target_os = "windows") {
                "frpc.exe"
            } else {
                "frpc"
            });
        self.download_binary(url.as_str(), &binary, &expected, &mut progress)?;
        progress(FrpcProgress::FetchingConfig);
        let config = self.config(id, &version)?;
        let config_path = root.join("config").join(format!("tunnel-{id}.ini"));
        write_private(&config_path, config.as_bytes())?;
        let log_path = root.join("logs").join(format!("tunnel-{id}.log"));
        fs::create_dir_all(log_path.parent().ok_or("日志目录无效")?)
            .map_err(|error| error.to_string())?;
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|error| format!("无法创建 frpc 日志：{error}"))?;
        let errors = log.try_clone().map_err(|error| error.to_string())?;
        let mut command = Command::new(&binary);
        command
            .arg("-c")
            .arg(&config_path)
            .current_dir(&root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(errors));
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        progress(FrpcProgress::Starting);
        let child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                let _ = fs::remove_file(&config_path);
                return Err(format!("启动 frpc 失败：{error}"));
            }
        };
        progress(FrpcProgress::Started);
        Ok(RunningTunnel {
            id,
            child,
            config_path,
            log_path,
        })
    }

    fn download_binary(
        &self,
        url: &str,
        target: &Path,
        expected: &str,
        progress: &mut impl FnMut(FrpcProgress),
    ) -> Result<(), String> {
        progress(FrpcProgress::CheckingCache);
        if target.is_file() && file_hash(target)? == expected {
            return Ok(());
        }
        fs::create_dir_all(target.parent().ok_or("frpc 下载目录无效")?)
            .map_err(|error| error.to_string())?;
        let temporary = target.with_extension("part");
        let result = (|| {
            progress(FrpcProgress::Downloading {
                downloaded: 0,
                total: None,
            });
            let mut response = self
                .http
                .get(url)
                .send()
                .and_then(reqwest::blocking::Response::error_for_status)
                .map_err(|error| format!("下载 frpc 失败：{error}"))?;
            let total = response.content_length().filter(|total| *total > 0);
            let mut downloaded = 0;
            let mut last_update = Instant::now();
            progress(FrpcProgress::Downloading { downloaded, total });
            let mut output = File::create(&temporary).map_err(|error| error.to_string())?;
            let mut hasher = Md5::new();
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
                downloaded += count as u64;
                if last_update.elapsed() >= Duration::from_millis(100) {
                    progress(FrpcProgress::Downloading { downloaded, total });
                    last_update = Instant::now();
                }
            }
            progress(FrpcProgress::Downloading { downloaded, total });
            progress(FrpcProgress::Verifying);
            output.flush().map_err(|error| error.to_string())?;
            if finish_hash(hasher) != expected {
                return Err("frpc 下载文件校验失败".to_owned());
            }
            if target.exists() {
                fs::remove_file(target).map_err(|error| error.to_string())?;
            }
            fs::rename(&temporary, target).map_err(|error| error.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(target, fs::Permissions::from_mode(0o700))
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

fn file_hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Md5::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(finish_hash(hasher))
}

fn finish_hash(hasher: Md5) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = hasher.finalize();
    let mut checksum = String::with_capacity(digest.len() * 2);
    for byte in digest {
        checksum.push(char::from(HEX[usize::from(byte >> 4)]));
        checksum.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    checksum
}

fn write_private(path: &Path, contents: &[u8]) -> Result<(), String> {
    fs::create_dir_all(path.parent().ok_or("配置目录无效")?).map_err(|error| error.to_string())?;
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|error| error.to_string())?;
    file.write_all(contents).map_err(|error| error.to_string())
}

fn platform_arch() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Ok("windows_amd64"),
        ("windows", "x86") => Ok("windows_386"),
        ("windows", "aarch64") => Ok("windows_arm64"),
        ("linux", "x86_64") => Ok("linux_amd64"),
        ("linux", "x86") => Ok("linux_386"),
        ("linux", "aarch64") => Ok("linux_arm64"),
        ("linux", "arm") => Ok("linux_armv7"),
        ("macos", "x86_64") => Ok("darwin_amd64"),
        ("macos", "aarch64") => Ok("darwin_arm64"),
        _ => Err("当前系统架构不支持内置 frpc".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::ClientDownloads;

    #[test]
    fn download_catalog_accepts_docker_links_without_checksums() {
        // The catalog mixes native binaries with Docker links, and other launcher categories
        // have their own artifact formats. Only a selected native binary requires a checksum.
        let downloads: ClientDownloads = serde_json::from_str(
            r#"{
                "unix": {
                    "archs": {
                        "docker_hub": {"url": "https://hub.docker.com/r/natfrp/launcher"}
                    }
                },
                "frpc": {
                    "ver": "0.51.0-sakura-14",
                    "archs": {
                        "docker_hub": {"url": "https://hub.docker.com/r/natfrp/frpc"},
                        "linux_amd64": {
                            "url": "https://nya.globalslb.net/natfrp/client/frpc/0.51.0-sakura-15/frpc_linux_amd64",
                            "hash": "9d9bde7f43fe7018ff9490a874ffc604"
                        }
                    }
                }
            }"#,
        )
        .unwrap();
        assert!(downloads.frpc.archs["docker_hub"].hash.is_none());
        assert_eq!(
            downloads.frpc.archs["linux_amd64"].hash.as_deref(),
            Some("9d9bde7f43fe7018ff9490a874ffc604")
        );
    }
}
