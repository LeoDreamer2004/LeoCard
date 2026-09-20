use super::super::{
    MAHJONG_HIGH_SHOWCASE_DELAY, MahjongAssets, MahjongTileMaterial, MahjongWinEffectTier,
    MahjongWinStageKind, MahjongWinStagePart, mahjong_win_effect_tier, mahjong_win_stage_start,
};
use super::{
    CenterWinHandContext, add_high_focus_rays, add_major_stage_decorations, render_center_win_hand,
};
use crate::app::presentation::spawn_node;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{MahjongHandResultView, MahjongSnapshot};

pub(super) struct WinStageContext<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub table: Entity,
    pub game: &'a MahjongSnapshot,
    pub result: &'a MahjongHandResultView,
    pub winner_index: usize,
    pub reveal_duration: f32,
    pub assets: &'a UiAssets,
    pub game_assets: &'a MahjongAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
}

pub(super) struct WinStagePartSpec {
    pub tier: MahjongWinEffectTier,
    pub reveal_duration: f32,
    pub start: f32,
    pub duration: f32,
    pub kind: MahjongWinStageKind,
    pub z_index: i32,
}

pub(super) fn add_win_stage_component(
    commands: &mut Commands,
    entity: Entity,
    spec: WinStagePartSpec,
) {
    commands.entity(entity).insert((
        MahjongWinStagePart {
            tier: spec.tier,
            reveal_duration: spec.reveal_duration,
            start: spec.start,
            duration: spec.duration,
            kind: spec.kind,
        },
        UiTransform::IDENTITY,
        Visibility::Hidden,
        ZIndex(spec.z_index),
        FocusPolicy::Pass,
    ));
}

pub(super) fn render_win_stage(context: WinStageContext<'_, '_, '_>) {
    let WinStageContext {
        commands,
        table,
        game,
        result,
        winner_index,
        reveal_duration,
        assets,
        game_assets,
        materials,
    } = context;
    let winner = &result.winners[winner_index];
    let tier = mahjong_win_effect_tier(winner);
    let start = mahjong_win_stage_start(result, winner_index);
    let duration = tier.presentation_duration();

    if tier == MahjongWinEffectTier::MajorFan {
        add_major_backdrop(commands, table, tier, reveal_duration, start, duration);
    }
    if tier == MahjongWinEffectTier::HighTotal {
        add_high_focus_rays(
            commands,
            table,
            reveal_duration,
            start + MAHJONG_HIGH_SHOWCASE_DELAY,
            duration - MAHJONG_HIGH_SHOWCASE_DELAY,
        );
    }
    if matches!(
        tier,
        MahjongWinEffectTier::HighTotal | MahjongWinEffectTier::MajorFan
    ) {
        let hand_delay = if tier == MahjongWinEffectTier::MajorFan {
            0.78
        } else {
            MAHJONG_HIGH_SHOWCASE_DELAY
        };
        render_center_win_hand(CenterWinHandContext {
            commands,
            table,
            game,
            winner,
            tier,
            reveal_duration,
            start: start + hand_delay,
            duration: duration - hand_delay,
            assets,
            game_assets,
            materials,
        });
    }
    if tier == MahjongWinEffectTier::MajorFan {
        add_major_stage_decorations(
            commands,
            table,
            winner,
            reveal_duration,
            start,
            duration,
            assets,
        );
    }
}

fn add_major_backdrop(
    commands: &mut Commands,
    table: Entity,
    tier: MahjongWinEffectTier,
    reveal_duration: f32,
    start: f32,
    duration: f32,
) {
    let backdrop = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        Some(Color::NONE),
    );
    let backdrop_start = start + 0.72;
    add_win_stage_component(
        commands,
        backdrop,
        WinStagePartSpec {
            tier,
            reveal_duration,
            start: backdrop_start,
            duration: (start + duration - backdrop_start).max(0.01),
            kind: MahjongWinStageKind::Backdrop,
            z_index: 90,
        },
    );
}
