//! GitHub Release 自动更新、下载进度和更新窗口。

use super::*;
use reqwest::blocking::Client;
use semver::Version;
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::io::{Read, Write};
use std::process::Command;
use std::time::{Duration, Instant};

pub(super) const GITHUB_REPOSITORY_URL: &str = "https://github.com/LeoDreamer2004/LeoCard";
const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/LeoDreamer2004/LeoCard/releases/latest";
const DOWNLOAD_BUFFER_SIZE: usize = 64 * 1024;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) enum UpdateState {
    #[default]
    Idle,
    Checking,
    Downloading {
        version: Version,
        downloaded: u64,
        total: Option<u64>,
    },
    UpToDate {
        version: Version,
    },
    Ready {
        version: Version,
        staged: PathBuf,
    },
    Failed(String),
}

#[derive(Resource, Default)]
pub(super) struct UpdateManager {
    pub(super) state: UpdateState,
    pub(super) dialog_open: bool,
    receiver: Option<Mutex<Receiver<UpdateEvent>>>,
}

impl UpdateManager {
    pub(super) fn begin_or_show(&mut self) {
        self.dialog_open = true;
        if matches!(
            self.state,
            UpdateState::Checking | UpdateState::Downloading { .. } | UpdateState::Ready { .. }
        ) {
            return;
        }
        if cfg!(debug_assertions) {
            self.state = UpdateState::Failed(
                "开发构建不会覆盖自身；请使用 GitHub Release 版本测试自动更新。".to_owned(),
            );
            return;
        }
        if !cfg!(any(target_os = "linux", target_os = "windows")) {
            self.state = UpdateState::Failed("当前平台暂不支持自动更新".to_owned());
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(Mutex::new(receiver));
        self.state = UpdateState::Checking;
        thread::spawn(move || {
            if let Err(error) = download_latest_release(&sender) {
                let _ = sender.send(UpdateEvent::Failed(error));
            }
        });
    }

    fn take_events(&mut self) -> (Vec<UpdateEvent>, bool) {
        let Some(receiver) = self.receiver.as_mut() else {
            return (Vec::new(), false);
        };
        let receiver = receiver
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut events = Vec::new();
        loop {
            match receiver.try_recv() {
                Ok(event) => events.push(event),
                Err(TryRecvError::Empty) => return (events, false),
                Err(TryRecvError::Disconnected) => return (events, true),
            }
        }
    }
}

#[derive(Debug)]
enum UpdateEvent {
    Available {
        version: Version,
        total: Option<u64>,
    },
    Progress {
        downloaded: u64,
        total: Option<u64>,
    },
    UpToDate {
        version: Version,
    },
    Ready {
        version: Version,
        staged: PathBuf,
    },
    Failed(String),
}

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

#[derive(Component)]
pub(super) struct UpdateProgressFill;

#[derive(Component)]
pub(super) struct UpdateStatusText;

#[derive(Component)]
pub(super) struct UpdateDetailText;

pub(super) fn poll_update_events(mut updater: ResMut<UpdateManager>, mut ui: ResMut<UiState>) {
    let (events, disconnected) = updater.take_events();
    let mut terminal = false;
    for event in events {
        match event {
            UpdateEvent::Available { version, total } => {
                updater.state = UpdateState::Downloading {
                    version,
                    downloaded: 0,
                    total,
                };
                ui.dirty = true;
            }
            UpdateEvent::Progress { downloaded, total } => {
                if let UpdateState::Downloading {
                    downloaded: current,
                    total: current_total,
                    ..
                } = &mut updater.state
                {
                    *current = downloaded;
                    *current_total = total;
                }
            }
            UpdateEvent::UpToDate { version } => {
                updater.state = UpdateState::UpToDate { version };
                updater.dialog_open = true;
                ui.dirty = true;
                terminal = true;
            }
            UpdateEvent::Ready { version, staged } => {
                updater.state = UpdateState::Ready { version, staged };
                // 即使用户把下载窗口放到了后台，完成时也要重新弹出。
                updater.dialog_open = true;
                ui.dirty = true;
                terminal = true;
            }
            UpdateEvent::Failed(error) => {
                updater.state = UpdateState::Failed(error);
                updater.dialog_open = true;
                ui.dirty = true;
                terminal = true;
            }
        }
    }
    if terminal {
        updater.receiver = None;
    } else if disconnected
        && matches!(
            updater.state,
            UpdateState::Checking | UpdateState::Downloading { .. }
        )
    {
        updater.state = UpdateState::Failed("更新任务意外中止，请重试".to_owned());
        updater.dialog_open = true;
        updater.receiver = None;
        ui.dirty = true;
    }
}

pub(super) fn sync_update_dialog(
    updater: Res<UpdateManager>,
    mut fills: Query<&mut Node, With<UpdateProgressFill>>,
    mut statuses: Query<&mut Text, (With<UpdateStatusText>, Without<UpdateDetailText>)>,
    mut details: Query<&mut Text, (With<UpdateDetailText>, Without<UpdateStatusText>)>,
) {
    if !updater.is_changed() {
        return;
    }
    let (status, detail, fraction) = update_display(&updater.state);
    for mut fill in &mut fills {
        fill.width = percent(fraction * 100.0);
    }
    for mut text in &mut statuses {
        text.0.clone_from(&status);
    }
    for mut text in &mut details {
        text.0.clone_from(&detail);
    }
}

pub(super) fn render_update_dialog(
    commands: &mut Commands,
    root: Entity,
    updater: &UpdateManager,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(Color::srgba(0.005, 0.015, 0.012, 0.76)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2300), FocusPolicy::Block));
    let modal = add_panel(
        commands,
        overlay,
        Node {
            width: px(560),
            max_width: percent(90),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Window,
        assets,
    );
    let title = if matches!(updater.state, UpdateState::Ready { .. }) {
        "更新下载完成"
    } else {
        "LeoCard 自动更新"
    };
    add_section_title(commands, modal, title, assets);

    let (status, detail, fraction) = update_display(&updater.state);
    let status_entity = add_text(commands, modal, status, 18.0, TEXT, assets);
    commands.entity(status_entity).insert(UpdateStatusText);
    let progress_track = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            height: px(18),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(9)),
            overflow: Overflow::clip(),
            ..default()
        },
        Some(HEADER_BG),
    );
    commands
        .entity(progress_track)
        .insert(BorderColor::all(BORDER));
    let fill = spawn_node(
        commands,
        progress_track,
        Node {
            width: percent(fraction * 100.0),
            height: percent(100),
            ..default()
        },
        Some(READY),
    );
    commands.entity(fill).insert(UpdateProgressFill);
    let detail_entity = add_text(commands, modal, detail, 13.0, MUTED, assets);
    commands.entity(detail_entity).insert(UpdateDetailText);

    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexEnd,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(10),
            row_gap: px(8),
            ..default()
        },
        None,
    );
    match updater.state {
        UpdateState::Ready { .. } => {
            add_action_button(
                commands,
                actions,
                "稍后重启",
                UiAction::HideUpdateDialog,
                ButtonKind::Secondary,
                assets,
            );
            add_green_update_button(
                commands,
                actions,
                "重启游戏并更新",
                UiAction::RestartToUpdate,
                assets,
            );
        }
        UpdateState::Failed(_) | UpdateState::UpToDate { .. } | UpdateState::Idle => {
            if matches!(updater.state, UpdateState::Failed(_)) {
                add_green_update_button(commands, actions, "重试", UiAction::StartUpdate, assets);
            }
            add_action_button(
                commands,
                actions,
                "关闭",
                UiAction::HideUpdateDialog,
                ButtonKind::Secondary,
                assets,
            );
        }
        UpdateState::Checking | UpdateState::Downloading { .. } => {
            add_action_button(
                commands,
                actions,
                "后台下载",
                UiAction::HideUpdateDialog,
                ButtonKind::Secondary,
                assets,
            );
        }
    }
}

#[derive(Component)]
pub(super) struct GitHubRepositoryButton;

pub(super) fn add_github_repository_button(
    commands: &mut Commands,
    parent: Entity,
    assets: &UiAssets,
) -> Entity {
    let normal = Color::srgb(0.31, 0.40, 0.48);
    let button = commands
        .spawn((
            Button,
            UiAction::OpenGitHubRepository,
            GitHubRepositoryButton,
            ButtonTint {
                normal,
                hovered: Color::srgb(0.45, 0.57, 0.67),
                pressed: Color::srgb(0.22, 0.30, 0.37),
            },
            Node {
                width: px(48),
                height: px(48),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(assets.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
            Name::new("打开 GitHub 仓库"),
        ))
        .id();
    commands.entity(parent).add_child(button);
    let mark = commands
        .spawn((
            Node {
                width: px(28),
                height: px(28),
                ..default()
            },
            ImageNode::new(assets.github_mark.clone()),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(mark);
    button
}

pub(super) fn open_github_repository() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let result = Command::new("explorer.exe")
        .arg(GITHUB_REPOSITORY_URL)
        .spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(GITHUB_REPOSITORY_URL).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = Command::new("xdg-open").arg(GITHUB_REPOSITORY_URL).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    return Err("当前系统不支持自动打开网页".to_owned());

    let mut child = result.map_err(|error| format!("无法打开 GitHub 仓库：{error}"))?;
    thread::spawn(move || match child.wait() {
        Ok(status) if !status.success() => warn!("系统浏览器未能打开 GitHub 仓库：{status}"),
        Err(error) => warn!("等待系统浏览器时出错：{error}"),
        Ok(_) => {}
    });
    Ok(())
}

pub(super) fn add_green_update_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: UiAction,
    assets: &UiAssets,
) -> Entity {
    let normal = READY;
    let entity = commands
        .spawn((
            Button,
            action,
            ButtonTint {
                normal,
                hovered: Color::srgb(0.48, 0.96, 0.64),
                pressed: Color::srgb(0.22, 0.66, 0.39),
            },
            Node {
                min_width: px(180),
                height: px(48),
                padding: UiRect::axes(px(20), px(8)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode::new(assets.primary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    add_text(commands, entity, label, 16.0, Color::WHITE, assets);
    entity
}

pub(super) fn settings_update_label(state: &UpdateState) -> &'static str {
    match state {
        UpdateState::Idle | UpdateState::UpToDate { .. } | UpdateState::Failed(_) => "检查并更新",
        UpdateState::Checking | UpdateState::Downloading { .. } => "查看更新进度",
        UpdateState::Ready { .. } => "重启并更新",
    }
}

fn update_display(state: &UpdateState) -> (String, String, f32) {
    match state {
        UpdateState::Idle => (
            "准备检查新版本".to_owned(),
            format!("当前版本 v{}", env!("CARGO_PKG_VERSION")),
            0.0,
        ),
        UpdateState::Checking => (
            "正在请求 GitHub 最新版本…".to_owned(),
            format!("当前版本 v{}", env!("CARGO_PKG_VERSION")),
            0.0,
        ),
        UpdateState::Downloading {
            version,
            downloaded,
            total,
        } => {
            let fraction = total
                .filter(|total| *total > 0)
                .map_or(0.0, |total| *downloaded as f32 / total as f32)
                .clamp(0.0, 1.0);
            let detail = total.map_or_else(
                || format!("已下载 {}", format_bytes(*downloaded)),
                |total| {
                    format!(
                        "已下载 {} / {}（{:.0}%）",
                        format_bytes(*downloaded),
                        format_bytes(total),
                        fraction * 100.0
                    )
                },
            );
            (format!("正在下载 LeoCard v{version}"), detail, fraction)
        }
        UpdateState::UpToDate { version } => (
            "当前已是最新版本".to_owned(),
            format!("已安装 LeoCard v{version}，无需更新。"),
            1.0,
        ),
        UpdateState::Ready { version, .. } => (
            format!("LeoCard v{version} 已下载并通过校验"),
            "点击“重启游戏并更新”完成安装；也可以稍后从游戏设置中重启。".to_owned(),
            1.0,
        ),
        UpdateState::Failed(error) => ("自动更新失败".to_owned(), error.clone(), 0.0),
    }
}

fn download_latest_release(sender: &mpsc::Sender<UpdateEvent>) -> Result<(), String> {
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
    Ok(format!("{:x}", digest.finalize()))
}

fn parse_sha256(contents: &str) -> Result<String, String> {
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
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|error| format!("无法设置更新文件的执行权限：{error}"))
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / MIB)
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_sha256sum_output() {
        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert_eq!(
            parse_sha256(&format!("{hash}  leocard-linux-x86_64\n")),
            Ok(hash.to_owned())
        );
    }

    #[test]
    fn rejects_malformed_sha256() {
        assert!(parse_sha256("1234  leocard").is_err());
        assert!(parse_sha256(&format!("{}g", "0".repeat(63))).is_err());
    }

    #[test]
    fn progress_display_is_clamped() {
        let state = UpdateState::Downloading {
            version: Version::new(1, 2, 3),
            downloaded: 150,
            total: Some(100),
        };
        let (_, detail, fraction) = update_display(&state);
        assert_eq!(fraction, 1.0);
        assert!(detail.contains("100%"));
    }
}
