use super::super::super::{
    MahjongAssets, MahjongCalligraphyKind, MahjongTileMaterial, MahjongWinEffect,
    mahjong_win_effect_tier, mahjong_win_reveal_duration,
};
use super::super::claim::MahjongSeatGeometry;
use super::{WinStageContext, mahjong_win_effect_visual, render_win_stage};
use crate::app::presentation::GameSummaryAnimation;
use crate::app::presentation::mahjong::projected_overlay;
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};
use leocard_ui::UiComposer;

pub(crate) struct MahjongWinVisuals<'a> {
    pub assets: &'a UiAssets,
    pub game_assets: &'a MahjongAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
}

pub(crate) fn render_mahjong_win_effects(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    animation: &GameSummaryAnimation,
    visuals: MahjongWinVisuals<'_>,
) {
    let geometry = MahjongSeatGeometry::new(own_seat);
    let MahjongPhaseView::Finished { result } = &game.phase else {
        return;
    };
    let reveal_duration = mahjong_win_reveal_duration(result);
    let MahjongWinVisuals {
        assets,
        game_assets,
        materials,
    } = visuals;
    for (winner_index, winner) in result.winners.iter().enumerate() {
        let tier = mahjong_win_effect_tier(winner);
        render_win_stage(WinStageContext {
            commands,
            table,
            game,
            result,
            winner_index,
            reveal_duration,
            assets,
            game_assets,
            materials,
        });
        let Some((scale, alpha, offset_y)) =
            mahjong_win_effect_visual(animation.elapsed, reveal_duration, tier)
        else {
            continue;
        };
        let Some(relative) = geometry.relative_player(game, winner.player) else {
            continue;
        };
        let position = MahjongSeatGeometry::river_anchor(relative);
        let overlay = projected_overlay(commands, table);
        let kind = if winner.from.is_none() {
            MahjongCalligraphyKind::SelfDraw
        } else {
            MahjongCalligraphyKind::Win
        };
        let holder = game_assets
            .calligraphy
            .spawn(commands, overlay, kind, position, alpha);
        commands.entity(holder).insert((
            MahjongWinEffect {
                tier,
                reveal_duration,
            },
            UiTransform {
                translation: Val2::px(0.0, offset_y),
                scale: Vec2::splat(scale),
                ..default()
            },
            ZIndex(95),
            Pickable::IGNORE,
        ));
    }
}
