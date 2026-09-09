use super::download::format_bytes;
use super::{UpdateEvent, UpdateManager, UpdateState};
use crate::app::{
    BORDER, ButtonKind, ButtonTint, HEADER_BG, MUTED, NavigationUiAction, PANEL, PanelSkin, READY,
    TEXT, UiAction, UiAssets, UiState, add_action_button, add_panel, add_section_title, add_text,
    spawn_node,
};
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

pub(crate) fn render_update_dialog(
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
                UiAction::Navigation(NavigationUiAction::HideUpdateDialog),
                ButtonKind::Secondary,
                assets,
            );
            add_green_update_button(
                commands,
                actions,
                "重启游戏并更新",
                UiAction::Navigation(NavigationUiAction::RestartToUpdate),
                assets,
            );
        }
        UpdateState::Failed(_) | UpdateState::UpToDate { .. } | UpdateState::Idle => {
            if matches!(updater.state, UpdateState::Failed(_)) {
                add_green_update_button(
                    commands,
                    actions,
                    "重试",
                    UiAction::Navigation(NavigationUiAction::StartUpdate),
                    assets,
                );
            }
            add_action_button(
                commands,
                actions,
                "关闭",
                UiAction::Navigation(NavigationUiAction::HideUpdateDialog),
                ButtonKind::Secondary,
                assets,
            );
        }
        UpdateState::Checking | UpdateState::Downloading { .. } => {
            add_action_button(
                commands,
                actions,
                "后台下载",
                UiAction::Navigation(NavigationUiAction::HideUpdateDialog),
                ButtonKind::Secondary,
                assets,
            );
        }
    }
}

#[derive(Component)]
pub(crate) struct GitHubRepositoryButton;

pub(crate) fn add_github_repository_button(
    commands: &mut Commands,
    parent: Entity,
    assets: &UiAssets,
) -> Entity {
    let normal = Color::srgb(0.31, 0.40, 0.48);
    let button = commands
        .spawn((
            Button,
            UiAction::Navigation(NavigationUiAction::OpenGitHubRepository),
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
            ImageNode::new(assets.controls.secondary_button.clone())
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
            ImageNode::new(assets.controls.github_mark.clone()),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(mark);
    button
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

pub(crate) fn add_green_update_button(
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
            ImageNode::new(assets.controls.primary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
        ))
        .id();
    commands.entity(parent).add_child(entity);
    add_text(commands, entity, label, 16.0, Color::WHITE, assets);
    entity
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
