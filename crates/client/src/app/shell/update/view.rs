use super::download::format_bytes;
use crate::app::shell::UpdateUiAction;

use super::{UpdateEvent, UpdateManager, UpdateState};
use crate::app::presentation::{MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, cozy_backdrop_color, cozy_panel_transform,
};
use crate::app::shell::{UiAction, UiState, add_cozy_button, add_cozy_panel};
use bevy::log::warn;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use std::process::Command;
use std::thread;

#[derive(Component)]
pub(crate) struct UpdateProgressFill;

#[derive(Component)]
pub(crate) struct UpdateStatusText;

#[derive(Component)]
pub(crate) struct UpdateDetailText;

pub(crate) fn poll_update_events(mut updater: ResMut<UpdateManager>, mut ui: ResMut<UiState>) {
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

pub(crate) fn sync_update_dialog(
    updater: Res<UpdateManager>,
    time: Res<Time>,
    mut fills: Query<&mut Node, With<UpdateProgressFill>>,
    mut statuses: Query<&mut Text, (With<UpdateStatusText>, Without<UpdateDetailText>)>,
    mut details: Query<&mut Text, (With<UpdateDetailText>, Without<UpdateStatusText>)>,
) {
    let indeterminate = matches!(
        updater.state,
        UpdateState::Checking | UpdateState::Downloading { total: None, .. }
    );
    if !updater.is_changed() && !indeterminate {
        return;
    }
    let (status, detail, fraction) = update_display(&updater.state);
    for mut fill in &mut fills {
        if indeterminate {
            fill.width = percent(28);
            fill.left = percent(((time.elapsed_secs() * 0.62) % 1.28 - 0.28) * 100.0);
        } else {
            fill.width = percent(fraction * 100.0);
            fill.left = px(0);
        }
    }
    if !updater.is_changed() {
        return;
    }
    for mut text in &mut statuses {
        text.0.clone_from(&status);
    }
    for mut text in &mut details {
        text.0.clone_from(&detail);
    }
}

pub(crate) fn render_update_dialog(
    commands: &mut Commands,
    root: Entity,
    updater: &UpdateManager,
    progress: f32,
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
        Some(cozy_backdrop_color(progress)),
    );
    commands.entity(overlay).insert((
        GlobalZIndex(2300),
        FocusPolicy::Block,
        CozyModalBackdrop(CozyModalKind::UpdateDialog),
    ));
    let modal = add_cozy_panel(
        commands,
        overlay,
        Node {
            width: px(560),
            max_width: percent(90),
            padding: UiRect::all(px(24)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: px(14),
            ..default()
        },
        assets,
    );
    commands.entity(modal).insert((
        CozyModalPanel(CozyModalKind::UpdateDialog),
        cozy_panel_transform(progress),
    ));
    let title = if matches!(updater.state, UpdateState::Ready { .. }) {
        "更新下载完成"
    } else {
        "LeoCard 自动更新"
    };
    add_text(commands, modal, title, 26.0, TEXT, assets);
    spawn_node(
        commands,
        modal,
        Node {
            width: px(96),
            height: px(2),
            ..default()
        },
        Some(Color::srgb(0.64, 0.59, 0.93)),
    );

    let (status, detail, fraction) = update_display(&updater.state);
    let status_entity = add_text(commands, modal, status, 18.0, TEXT, assets);
    commands.entity(status_entity).insert(UpdateStatusText);
    let progress_track = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            height: px(14),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            overflow: Overflow::clip(),
            ..default()
        },
        Some(Color::srgb(0.15, 0.15, 0.18)),
    );
    commands
        .entity(progress_track)
        .insert(BorderColor::all(Color::srgba(0.62, 0.60, 0.69, 0.52)));
    let fill = spawn_node(
        commands,
        progress_track,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(
                if matches!(
                    updater.state,
                    UpdateState::Checking | UpdateState::Downloading { total: None, .. }
                ) {
                    28.0
                } else {
                    fraction * 100.0
                },
            ),
            height: percent(100),
            ..default()
        },
        Some(Color::srgb(0.54, 0.49, 0.88)),
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
            add_cozy_button(
                commands,
                actions,
                "稍后重启",
                UiAction::Update(UpdateUiAction::HideUpdateDialog),
                assets,
                px(120),
                42.0,
            );
            add_cozy_button(
                commands,
                actions,
                "重启游戏并更新",
                UiAction::Update(UpdateUiAction::RestartToUpdate),
                assets,
                px(185),
                42.0,
            );
        }
        UpdateState::Failed(_) | UpdateState::UpToDate { .. } | UpdateState::Idle => {
            if matches!(updater.state, UpdateState::Failed(_)) {
                add_cozy_button(
                    commands,
                    actions,
                    "重试",
                    UiAction::Update(UpdateUiAction::StartUpdate),
                    assets,
                    px(100),
                    42.0,
                );
            }
            add_cozy_button(
                commands,
                actions,
                "关闭",
                UiAction::Update(UpdateUiAction::HideUpdateDialog),
                assets,
                px(100),
                42.0,
            );
        }
        UpdateState::Checking | UpdateState::Downloading { .. } => {
            add_cozy_button(
                commands,
                actions,
                if matches!(updater.state, UpdateState::Checking) {
                    "后台检查"
                } else {
                    "后台下载"
                },
                UiAction::Update(UpdateUiAction::HideUpdateDialog),
                assets,
                px(120),
                42.0,
            );
        }
    }
}

pub(crate) fn open_github_repository() -> Result<(), String> {
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

pub(crate) fn settings_update_label(state: &UpdateState) -> &'static str {
    match state {
        UpdateState::Idle | UpdateState::UpToDate { .. } | UpdateState::Failed(_) => "检查并更新",
        UpdateState::Checking | UpdateState::Downloading { .. } => "查看更新进度",
        UpdateState::Ready { .. } => "重启并更新",
    }
}

pub(super) fn update_display(state: &UpdateState) -> (String, String, f32) {
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

const GITHUB_REPOSITORY_URL: &str = "https://github.com/LeoDreamer2004/LeoCard";
