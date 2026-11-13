//! Score deltas and rolling totals for matches with multiple hands.
use super::super::{
    MahjongAssets, mahjong_status_score_angle, mahjong_status_score_position,
    render_round_status_with_scores,
};
use super::*;
use crate::app::presentation::mahjong::projected_overlay;
use crate::app::presentation::{DANGER, MUTED, READY, TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::{MahjongHandResultView, MahjongPhaseView, MahjongSnapshot};
use leocard_ui::{UiComposer, spawn_node};
use std::array;

pub(in crate::app::games::mahjong) fn render_round_status(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    elapsed: f32,
    assets: &UiAssets,
    game_assets: &MahjongAssets,
) {
    let rolling_scores: Option<[MahjongScoreValue; 4]> = match &game.phase {
        MahjongPhaseView::Finished { result } => SettlementTimeline::new(result)
            .scores
            .map(|timing| array::from_fn(|slot| MahjongScoreValue::new(result, slot, timing.roll))),
        _ => None,
    };
    // The finished snapshot already contains the final totals. Keep the table on
    // the same timeline as the settlement overlay, including after UI rebuilds.
    let scores = rolling_scores.map_or(game.match_scores, |scores| {
        scores.map(|score| score.at(elapsed))
    });
    let texts = render_round_status_with_scores(
        commands,
        table,
        game,
        own_seat,
        &scores,
        assets,
        game_assets,
    );
    if let Some(scores) = rolling_scores {
        for player in &game.players {
            let relative = usize::from((player.seat.0 + 4 - own_seat) % 4);
            commands
                .entity(texts[relative])
                .insert(scores[usize::from(player.player.id.0)]);
        }
    }
}

pub(super) struct ScoreStageView<'a> {
    pub(super) game: &'a MahjongSnapshot,
    pub(super) result: &'a MahjongHandResultView,
    pub(super) assets: &'a UiAssets,
    pub(super) game_assets: &'a MahjongAssets,
    pub(super) elapsed: f32,
    pub(super) timing: ScoreTiming,
    pub(super) end: f32,
}

impl ScoreStageView<'_> {
    pub(super) fn render(self, commands: &mut UiComposer, table: Entity) {
        let Self {
            game,
            result,
            assets,
            game_assets,
            elapsed,
            timing,
            end,
        } = self;
        let own_seat = game
            .players
            .iter()
            .find(|player| player.player.id == game.you)
            .map_or(0, |player| player.seat.0);

        let score_stage = spawn_node(
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
            None,
        );
        commands.entity(score_stage).insert((
            MahjongScoreStage {
                start: timing.start,
                end,
            },
            GlobalZIndex(1200),
            if elapsed >= timing.start && elapsed < end {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));

        // 复刻牌桌中央的方位牌，让遮罩只留下这一块亮区。四面数值从结算前
        // 的分数开始，沿各自座位的朝向接收增减，再滚到结算后的分数。
        let layout_scores = array::from_fn(|slot| {
            let to = result.match_scores[slot];
            let from = to.saturating_sub(result.deltas[slot]);
            if from.to_string().len() > to.to_string().len() {
                from
            } else {
                to
            }
        });
        let score_texts = render_round_status_with_scores(
            commands,
            score_stage,
            game,
            own_seat,
            &layout_scores,
            assets,
            game_assets,
        );
        let overlay = projected_overlay(commands, score_stage);
        for (relative, score_text) in score_texts.into_iter().enumerate() {
            let Some(player) = game
                .players
                .iter()
                .find(|player| usize::from((player.seat.0 + 4 - own_seat) % 4) == relative)
            else {
                continue;
            };
            let slot = usize::from(player.player.id.0);
            let delta = result.deltas[slot];
            let score = MahjongScoreValue::new(result, slot, timing.roll);
            commands.entity(score_text).insert((
                Text(score.at(elapsed).to_string()),
                score,
                TextColor(TEXT),
            ));
            let position = mahjong_status_score_position(relative as u8) - Vec2::new(7.0, 9.0);
            let origin = match relative {
                0 => Vec2::new(0.0, 38.0),
                1 => Vec2::new(38.0, 0.0),
                2 => Vec2::new(0.0, -38.0),
                _ => Vec2::new(-38.0, 0.0),
            };
            let rotation = mahjong_status_score_angle(relative as u8);
            let color = if delta > 0 {
                READY
            } else if delta < 0 {
                DANGER
            } else {
                MUTED
            };
            let flight_progress = delta_progress(elapsed, timing.flight);
            let flying = add_text(
                commands,
                overlay,
                format!("{delta:+}"),
                18.0,
                color.with_alpha(1.0 - flight_progress.powi(3)),
                assets,
            );
            commands.entity(flying).insert((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(position.x),
                    top: px(position.y),
                    ..default()
                },
                MahjongFlyingDelta {
                    appear_at: timing.start + DELTA_APPEAR_DELAY,
                    start: timing.flight,
                    color,
                    origin,
                    rotation,
                },
                delta_transform(flight_progress, origin, rotation),
                if elapsed >= timing.start + DELTA_APPEAR_DELAY && flight_progress < 1.0 {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                },
                Pickable::IGNORE,
            ));
        }
    }
}
