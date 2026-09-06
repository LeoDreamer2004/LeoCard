use super::UpdateEvent;
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub(super) fn download_latest_release(sender: &mpsc::Sender<UpdateEvent>) -> Result<(), String> {
    let client = Client::builder()
        .user_agent(format!("LeoCard/{}", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(30 * 60))
        .build()
        .map_err(|error| format!("无法初始化 HTTPS 客户端：{error}"))?;
    let release = client
        .get(LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("无法获取 GitHub 最新版本：{error}"))?
        .json::<GitHubRelease>()
        .map_err(|error| format!("无法解析 GitHub Release：{error}"))?;

    let version_text = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let latest = Version::parse(version_text)
        .map_err(|error| format!("Release 标签 {} 不是有效版本号：{error}", release.tag_name))?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| format!("当前程序版本号无效：{error}"))?;
    if latest <= current {
        sender
            .send(UpdateEvent::UpToDate { version: current })
            .map_err(|_| "更新窗口已经关闭".to_owned())?;
        return Ok(());
    }

    let binary_name = platform_asset_name()?;
    let binary = release
        .assets
        .iter()
        .find(|asset| asset.name == binary_name)
        .ok_or_else(|| format!("Release 中缺少 {binary_name}"))?;
    let checksum_name = format!("{binary_name}.sha256");
    let checksum = release
        .assets
        .iter()
        .find(|asset| asset.name == checksum_name)
        .ok_or_else(|| format!("Release 中缺少 {checksum_name}"))?;
    let expected_checksum = download_checksum(&client, checksum)?;
    let total = Some(binary.size).filter(|size| *size > 0);
    sender
        .send(UpdateEvent::Available {
            version: latest.clone(),
            total,
        })
        .map_err(|_| "更新窗口已经关闭".to_owned())?;

    let staged = staged_update_path(&latest)?;
    let part = partial_update_path(&staged);
    let result = download_binary(&client, binary, &part, total, sender).and_then(|actual| {
        if actual != expected_checksum {
            return Err(format!(
                "下载文件校验失败：期望 {expected_checksum}，实际 {actual}"
            ));
        }
        set_executable_permissions(&part)?;
        if staged.exists() {
            fs::remove_file(&staged).map_err(|error| format!("无法清理旧更新文件：{error}"))?;
        }
        fs::rename(&part, &staged).map_err(|error| format!("无法保存更新文件：{error}"))?;
        Ok(())
    });
    if let Err(error) = result {
        let _ = fs::remove_file(part);
        return Err(error);
    }
    sender
        .send(UpdateEvent::Ready {
            version: latest,
            staged,
        })
        .map_err(|_| "更新窗口已经关闭".to_owned())?;
    Ok(())
}

fn download_checksum(client: &Client, asset: &GitHubAsset) -> Result<String, String> {
    if asset.size > 16 * 1024 {
        return Err("Release 校验文件异常过大".to_owned());
    }
    let text = client
        .get(&asset.browser_download_url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("无法下载 SHA-256 校验文件：{error}"))?
        .text()
        .map_err(|error| format!("无法读取 SHA-256 校验文件：{error}"))?;
    parse_sha256(&text)
}

fn download_binary(
    client: &Client,
    asset: &GitHubAsset,
    path: &Path,
    expected_total: Option<u64>,
    sender: &mpsc::Sender<UpdateEvent>,
) -> Result<String, String> {
    let mut response = client
        .get(&asset.browser_download_url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| format!("无法下载更新：{error}"))?;
    let total = response.content_length().or(expected_total);
    let mut file = fs::File::create(path)
        .map_err(|error| format!("无法在程序目录创建更新文件（请检查写入权限）：{error}"))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; DOWNLOAD_BUFFER_SIZE];
    let mut downloaded = 0_u64;
    let mut last_progress = Instant::now();
    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|error| format!("下载更新时连接中断：{error}"))?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|error| format!("写入更新文件失败：{error}"))?;
        digest.update(&buffer[..read]);
        downloaded = downloaded.saturating_add(read as u64);
        if last_progress.elapsed() >= PROGRESS_INTERVAL {
            let _ = sender.send(UpdateEvent::Progress { downloaded, total });
            last_progress = Instant::now();
        }
    }
    file.sync_all()
        .map_err(|error| format!("保存更新文件失败：{error}"))?;
    let _ = sender.send(UpdateEvent::Progress { downloaded, total });
    if expected_total.is_some_and(|expected| expected != downloaded) {
        return Err(format!(
            "下载大小不完整：期望 {}，实际 {}",
            format_bytes(expected_total.unwrap_or_default()),
            format_bytes(downloaded)
        ));
    }
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = digest.finalize();
    let mut checksum = String::with_capacity(digest.len() * 2);
    for byte in digest {
        checksum.push(char::from(HEX[usize::from(byte >> 4)]));
        checksum.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(checksum)
}

pub(super) fn parse_sha256(contents: &str) -> Result<String, String> {
    let checksum = contents
        .split_whitespace()
        .next()
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "SHA-256 校验文件为空".to_owned())?;
    if checksum.len() != 64 || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("SHA-256 校验文件格式错误".to_owned());
    }
    Ok(checksum)
}

fn platform_asset_name() -> Result<&'static str, String> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return Ok("leocard-linux-x86_64");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return Ok("leocard-windows-x86_64.exe");
    #[allow(unreachable_code)]
    Err("当前系统没有对应的 GitHub Release 文件".to_owned())
}

fn staged_update_path(version: &Version) -> Result<PathBuf, String> {
    let executable =
        std::env::current_exe().map_err(|error| format!("无法定位当前程序：{error}"))?;
    let file_name = executable
        .file_name()
        .ok_or_else(|| "当前程序路径没有文件名".to_owned())?;
    let mut staged_name = OsString::from(".");
    staged_name.push(file_name);
    staged_name.push(format!(".update-{version}"));
    Ok(executable.with_file_name(staged_name))
}

fn partial_update_path(staged: &Path) -> PathBuf {
    let mut name = staged
        .file_name()
        .map_or_else(|| OsString::from("leocard-update"), OsString::from);
    name.push(".part");
    staged.with_file_name(name)
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> Result<(), String> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|error| format!("无法设置更新文件的执行权限：{error}"))
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

pub(super) fn format_bytes(bytes: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / MIB)
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/LeoDreamer2004/LeoCard/releases/latest";
const DOWNLOAD_BUFFER_SIZE: usize = 64 * 1024;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
