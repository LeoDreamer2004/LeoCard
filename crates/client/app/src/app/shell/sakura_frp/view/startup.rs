use super::super::actions::SakuraUiAction;
use super::super::state::SakuraState;
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, ModalAnimations, UiAction, UiState,
    add_cozy_button, add_cozy_panel, advance_modal, cozy_backdrop_color, cozy_panel_transform,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_sakura_frp::FrpcProgress;
use leocard_ui::{ProgressBar, UiComposer, spawn_node};

#[derive(Component)]
pub(in super::super) struct StartupStatusText;

#[derive(Component)]
pub(in super::super) struct StartupDetailText;

#[derive(Component)]
pub(in super::super) struct StartupProgressFill;

pub(in super::super) fn advance_startup_modal(
    time: Res<Time>,
    mut state: ResMut<SakuraState>,
    mut animations: ResMut<ModalAnimations>,
    mut ui: ResMut<UiState>,
) {
    let Some(startup) = &mut state.startup else {
        animations.set(CozyModalKind::SakuraStartup, 0.0);
        return;
    };
    advance_modal(&mut startup.animation, startup.open, time.delta_secs());
    animations.set(CozyModalKind::SakuraStartup, startup.animation);
    if !startup.open && startup.animation == 0.0 {
        state.startup = None;
        ui.dirty = true;
    }
}

pub(in super::super) fn sync_startup_dialog(
    state: Res<SakuraState>,
    mut bars: Query<&mut ProgressBar, With<StartupProgressFill>>,
    mut statuses: Query<&mut Text, (With<StartupStatusText>, Without<StartupDetailText>)>,
    mut details: Query<&mut Text, (With<StartupDetailText>, Without<StartupStatusText>)>,
) {
    let Some(startup) = &state.startup else {
        return;
    };
    let (status, detail, fraction) = startup_display(&startup.stage);
    for mut bar in &mut bars {
        if bar.fraction != fraction {
            bar.fraction = fraction;
        }
    }
    for mut text in &mut statuses {
        if text.0 != status {
            text.0 = status.to_owned();
        }
    }
    for mut text in &mut details {
        if text.0 != detail {
            text.0.clone_from(&detail);
        }
    }
}

pub(crate) fn render_startup_dialog(
    commands: &mut UiComposer,
    root: Entity,
    state: &SakuraState,
    assets: &UiAssets,
) {
    let Some(startup) = &state.startup else {
        return;
    };
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
        Some(cozy_backdrop_color(startup.animation)),
    );
    commands.entity(overlay).insert((
        GlobalZIndex(2200),
        Pickable::default(),
        CozyModalBackdrop(CozyModalKind::SakuraStartup),
    ));
    let panel = add_cozy_panel(
        commands,
        overlay,
        Node {
            width: px(520),
            max_width: percent(90),
            padding: UiRect::all(px(26)),
            flex_direction: FlexDirection::Column,
            row_gap: px(16),
            ..default()
        },
        assets,
    );
    commands.entity(panel).insert((
        CozyModalPanel(CozyModalKind::SakuraStartup),
        cozy_panel_transform(startup.animation),
    ));
    add_text(commands, panel, "隧道启动", 25.0, TEXT, assets);
    add_text(commands, panel, &startup.name, 18.0, MUTED, assets);
    let (status, detail, fraction) = startup_display(&startup.stage);
    let status = add_text(commands, panel, status, 18.0, TEXT, assets);
    commands.entity(status).insert(StartupStatusText);
    let fill = ProgressBar::spawn(commands, panel, fraction);
    commands.entity(fill).insert(StartupProgressFill);
    let detail = add_text(commands, panel, detail, 14.0, MUTED, assets);
    commands.entity(detail).insert(StartupDetailText);
    if startup.stage == FrpcProgress::Started {
        add_cozy_button(
            commands,
            panel,
            "完成",
            UiAction::Sakura(SakuraUiAction::CloseStartup),
            assets,
            percent(100),
            42.0,
        );
    }
}

fn startup_display(stage: &FrpcProgress) -> (&'static str, String, Option<f32>) {
    let status = match stage {
        FrpcProgress::ResolvingDownload => "正在获取 frpc 下载信息…",
        FrpcProgress::CheckingCache => "正在检查本地 frpc…",
        FrpcProgress::Downloading { downloaded, total } => {
            let fraction = total
                .filter(|total| *total > 0)
                .map(|total| (*downloaded as f32 / total as f32).clamp(0.0, 1.0));
            let detail = match (total, fraction) {
                (Some(total), Some(fraction)) => format!(
                    "{:.1} / {:.1} MiB（{:.0}%）",
                    *downloaded as f64 / 1_048_576.0,
                    *total as f64 / 1_048_576.0,
                    fraction * 100.0,
                ),
                _ => format!("已下载 {:.1} MiB", *downloaded as f64 / 1_048_576.0),
            };
            return ("正在下载 frpc…", detail, fraction);
        }
        FrpcProgress::Verifying => "正在校验下载文件…",
        FrpcProgress::FetchingConfig => "正在获取隧道配置…",
        FrpcProgress::Starting => "正在启动 frpc…",
        FrpcProgress::Started => "隧道已在本机启动",
    };
    (
        status,
        String::new(),
        (*stage == FrpcProgress::Started).then_some(1.0),
    )
}
