use super::super::{
    MahjongAssets, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual, MahjongWinEffectTier,
    MahjongWinStageKind, add_mahjong_tile_material,
};
use super::{WinStagePartSpec, add_win_stage_component, mahjong_win_effect_color};
use crate::app::presentation::{add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{MahjongSnapshot, MahjongWinView};

pub(super) struct CenterWinHandContext<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub table: Entity,
    pub game: &'a MahjongSnapshot,
    pub winner: &'a MahjongWinView,
    pub tier: MahjongWinEffectTier,
    pub reveal_duration: f32,
    pub start: f32,
    pub duration: f32,
    pub assets: &'a UiAssets,
    pub game_assets: &'a MahjongAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
}

pub(super) fn render_center_win_hand(context: CenterWinHandContext<'_, '_, '_>) {
    let CenterWinHandContext {
        commands,
        table,
        game,
        winner,
        tier,
        reveal_duration,
        start,
        duration,
        assets,
        game_assets,
        materials,
    } = context;
    let Some(player) = game
        .players
        .iter()
        .find(|player| player.id == winner.player)
    else {
        return;
    };
    let Some(revealed) = &player.revealed_hand else {
        return;
    };
    let major = tier == MahjongWinEffectTier::MajorFan;
    let panel = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(if major { 252 } else { 248 }),
            height: px(if major { 176 } else { 184 }),
            padding: UiRect::axes(px(20), px(18)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(12),
            overflow: Overflow::visible(),
            ..default()
        },
        Some(Color::NONE),
    );
    let content = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(12),
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    add_text(
        commands,
        content,
        format!("{} 的和牌", player.name),
        18.0,
        mahjong_win_effect_color(tier, false, 1.0),
        assets,
    );
    let row = spawn_node(
        commands,
        content,
        Node {
            height: px(76),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Row,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    let mut removed_winning_tile = false;
    for (index, tile) in revealed.iter().enumerate() {
        if !removed_winning_tile && *tile == winner.winning_tile {
            removed_winning_tile = true;
            continue;
        }
        add_mahjong_tile_material(
            commands,
            row,
            MahjongTileVisual {
                kind: Some(tile.kind()),
                size: MahjongTileSize::OwnMeld,
                index,
                highlighted: false,
                deal: None,
                relative: 0,
            },
            game_assets,
            materials,
        );
    }
    let gap = spawn_node(
        commands,
        row,
        Node {
            width: px(18),
            min_width: px(18),
            ..default()
        },
        None,
    );
    commands.entity(gap).insert(FocusPolicy::Pass);
    add_mahjong_tile_material(
        commands,
        row,
        MahjongTileVisual {
            kind: Some(winner.winning_tile.kind()),
            size: MahjongTileSize::OwnMeld,
            index: revealed.len(),
            highlighted: true,
            deal: None,
            relative: 0,
        },
        game_assets,
        materials,
    );
    if !major {
        add_win_stage_component(
            commands,
            panel,
            WinStagePartSpec {
                tier,
                reveal_duration,
                start,
                duration,
                kind: MahjongWinStageKind::Backdrop,
                z_index: 101,
            },
        );
    }
    add_win_stage_component(
        commands,
        if major { content } else { row },
        WinStagePartSpec {
            tier,
            reveal_duration,
            start,
            duration,
            kind: MahjongWinStageKind::Hand,
            z_index: 102,
        },
    );
}
