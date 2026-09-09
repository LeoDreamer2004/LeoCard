use super::super::claim::MahjongSeatGeometry;
use super::super::{
    MahjongAssets, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual, MahjongWinEffectTier,
    MahjongWinStageKind, MahjongWinStagePart, add_mahjong_tile_material, mahjong_win_effect_tier,
    mahjong_win_stage_start,
};
use super::{
    CenterWinHandContext, add_high_focus_rays, add_major_stage_decorations, render_center_win_hand,
};
use crate::app::presentation::spawn_node;
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{MahjongHandResultView, MahjongSnapshot, MahjongWinView};

pub(super) struct WinStageContext<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub table: Entity,
    pub game: &'a MahjongSnapshot,
    pub own_seat: u8,
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
        own_seat,
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
        add_high_focus_rays(commands, table, reveal_duration, start, duration);
    }
    if let Some(anchor) = emphasized_tile_anchor(game, own_seat, winner, tier) {
        add_emphasized_tile(
            commands,
            table,
            anchor,
            winner,
            tier,
            reveal_duration,
            start,
            duration,
            game_assets,
            materials,
        );
    }
    if matches!(
        tier,
        MahjongWinEffectTier::HighTotal | MahjongWinEffectTier::MajorFan
    ) {
        let hand_delay = if tier == MahjongWinEffectTier::MajorFan {
            0.78
        } else {
            0.10
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

fn emphasized_tile_anchor(
    game: &MahjongSnapshot,
    own_seat: u8,
    winner: &MahjongWinView,
    tier: MahjongWinEffectTier,
) -> Option<Vec2> {
    if tier == MahjongWinEffectTier::HighTotal {
        return None;
    }
    let geometry = MahjongSeatGeometry::new(own_seat);
    if let Some(source) = winner.from {
        return geometry
            .relative_player(game, source)
            .map(MahjongSeatGeometry::river_anchor);
    }
    geometry
        .relative_player(game, winner.player)
        .map(|relative| match relative {
            0 => Vec2::new(1005.0, 642.0),
            1 => Vec2::new(1080.0, 450.0),
            2 => Vec2::new(445.0, 92.0),
            _ => Vec2::new(200.0, 220.0),
        })
}

#[allow(clippy::too_many_arguments)]
fn add_emphasized_tile(
    commands: &mut Commands,
    table: Entity,
    anchor: Vec2,
    winner: &MahjongWinView,
    tier: MahjongWinEffectTier,
    reveal_duration: f32,
    start: f32,
    duration: f32,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let holder = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(anchor.x - 50.0),
            top: px(anchor.y - 70.0),
            width: px(100),
            height: px(140),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    add_mahjong_tile_material(
        commands,
        holder,
        MahjongTileVisual {
            kind: Some(winner.winning_tile.kind()),
            size: MahjongTileSize::OwnMeld,
            index: 0,
            highlighted: true,
            deal: None,
            relative: 0,
        },
        game_assets,
        materials,
    );
    add_win_stage_component(
        commands,
        holder,
        WinStagePartSpec {
            tier,
            reveal_duration,
            start,
            duration: duration.min(if tier == MahjongWinEffectTier::MajorFan {
                0.84
            } else {
                duration
            }),
            kind: MahjongWinStageKind::WinningTile,
            z_index: 104,
        },
    );
}
