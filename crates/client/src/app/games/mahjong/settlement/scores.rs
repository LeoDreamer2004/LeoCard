//! Score deltas and rolling totals for matches with multiple hands.

use super::super::{MahjongAssets, render_round_status_with_scores};
use super::*;
use crate::app::presentation::{DANGER, MUTED, READY, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::{MahjongHandResultView, MahjongSnapshot};
use std::{array, f32::consts};

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
    pub(super) fn render(self, commands: &mut Commands, table: Entity) {
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
            .find(|player| player.id == game.you)
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
        for (relative, score_text) in score_texts.into_iter().enumerate() {
            let Some(player) = game
                .players
                .iter()
                .find(|player| usize::from((player.seat.0 + 4 - own_seat) % 4) == relative)
            else {
                continue;
            };
            let slot = usize::from(player.id.0);
            let total = result.match_scores[slot];
            let delta = result.deltas[slot];
            let from = total.saturating_sub(delta);
            commands.entity(score_text).insert((
                Text(score_at(elapsed, from, total, timing.roll).to_string()),
                MahjongScoreValue {
                    from,
                    to: total,
                    start: timing.roll,
                },
                TextColor(TEXT),
            ));
            let (left, top, origin, rotation) = match relative {
                0 => (637.0, 359.0, Vec2::new(0.0, 38.0), 0.0),
                1 => (692.0, 317.0, Vec2::new(38.0, 0.0), -consts::FRAC_PI_2),
                2 => (637.0, 281.0, Vec2::new(0.0, -38.0), consts::PI),
                _ => (575.0, 317.0, Vec2::new(-38.0, 0.0), consts::FRAC_PI_2),
            };
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
                score_stage,
                format!("{delta:+}"),
                18.0,
                color.with_alpha(1.0 - flight_progress.powi(3)),
                assets,
            );
            commands.entity(flying).insert((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
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
