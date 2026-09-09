use super::super::claim::MahjongSeatGeometry;
use super::super::{
    MahjongAssets, MahjongTileMaterial, MahjongWinEffect, MahjongWinEffectText,
    MahjongWinEffectTier, mahjong_win_effect_tier, mahjong_win_reveal_duration,
};
use super::{
    WinStageContext, add_win_aura, mahjong_win_effect_color, mahjong_win_effect_visual,
    render_win_stage,
};
use crate::app::presentation::{GameSummaryAnimation, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};

pub(crate) struct MahjongWinVisuals<'a> {
    pub assets: &'a UiAssets,
    pub game_assets: &'a MahjongAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
}

pub(crate) fn render_mahjong_win_effects(
    commands: &mut Commands,
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
            own_seat,
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
        let holder = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x - 55.0),
                top: px(position.y - 43.0),
                width: px(110),
                height: px(86),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        add_win_aura(commands, holder, tier, reveal_duration);
        let self_draw = winner.from.is_none();
        let text = add_text(
            commands,
            holder,
            if self_draw { "自摸" } else { "和" },
            win_label_size(tier, self_draw),
            mahjong_win_effect_color(tier, false, alpha),
            assets,
        );
        commands.entity(text).insert((
            MahjongWinEffectText {
                tier,
                reveal_duration,
            },
            TextShadow {
                offset: Vec2::new(2.0, 4.0),
                color: Color::BLACK.with_alpha(0.76 * alpha),
            },
            ZIndex(1),
        ));
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
            FocusPolicy::Pass,
        ));
    }
}

fn win_label_size(tier: MahjongWinEffectTier, self_draw: bool) -> f32 {
    match (tier, self_draw) {
        (MahjongWinEffectTier::Normal, true) => 42.0,
        (MahjongWinEffectTier::HighTotal, true) => 46.0,
        (MahjongWinEffectTier::MajorFan, true) => 50.0,
        (MahjongWinEffectTier::Normal, false) => 54.0,
        (MahjongWinEffectTier::HighTotal, false) => 60.0,
        (MahjongWinEffectTier::MajorFan, false) => 66.0,
    }
}
