//! Settlement timing and visual animation components.

use super::super::{MahjongUiState, mahjong_win_reveal_duration};
use super::*;
use crate::app::presentation::{
    GameSummaryAnimation, SUMMARY_ACTIONS_EXTRA_DELAY, SUMMARY_ROW_INTERVAL,
    SUMMARY_ROW_START_DELAY, SummaryDescriptor, summary_row_progress,
};
use bevy::prelude::*;
use leocard_protocol::{MahjongHandResultView, MahjongPhaseView, MahjongSnapshot};

pub(crate) fn mahjong_fan_pause_at(result: &MahjongHandResultView) -> Option<f32> {
    (!result.winners.is_empty())
        .then(|| SettlementTimeline::new(result).pages_end - PAGE_FADE_DURATION - 0.04)
}

pub(super) fn score_at(elapsed: f32, from: i32, to: i32, start: f32) -> i32 {
    let progress = ((elapsed - start) / SCORE_ROLL_DURATION).clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - progress).powi(3);
    (from as f32 + (to as f32 - from as f32) * eased).round() as i32
}

pub(super) fn delta_progress(elapsed: f32, start: f32) -> f32 {
    ((elapsed - start) / DELTA_FLIGHT_DURATION).clamp(0.0, 1.0)
}

pub(super) fn delta_transform(progress: f32, origin: Vec2, rotation: f32) -> UiTransform {
    let eased = progress * progress * (3.0 - 2.0 * progress);
    UiTransform {
        translation: Val2::px(origin.x * (1.0 - eased), origin.y * (1.0 - eased)),
        scale: Vec2::splat(1.0 - 0.38 * eased),
        rotation: Rot2::radians(rotation),
    }
}

#[derive(Component)]
pub(in crate::app::games::mahjong) struct MahjongFanPage {
    pub(super) start: f32,
    pub(super) end: f32,
}

#[derive(Component)]
pub(in crate::app::games::mahjong) struct MahjongFanPageCurtain {
    pub(super) end: f32,
}

#[derive(Component)]
pub(in crate::app::games::mahjong) struct MahjongFanEntry {
    pub(super) delay: f32,
    pub(super) end: f32,
}

#[derive(Component)]
pub(in crate::app::games::mahjong) struct MahjongScoreStage {
    pub(super) start: f32,
    pub(super) end: f32,
}

#[derive(Component)]
pub(in crate::app::games::mahjong) struct MahjongScoreShade {
    pub(super) start: f32,
}

#[derive(Component)]
pub(in crate::app::games::mahjong) struct MahjongScoreValue {
    pub(super) from: i32,
    pub(super) to: i32,
    pub(super) start: f32,
}

#[derive(Component)]
pub(in crate::app::games::mahjong) struct MahjongFlyingDelta {
    pub(super) appear_at: f32,
    pub(super) start: f32,
    pub(super) color: Color,
    pub(super) origin: Vec2,
    pub(super) rotation: f32,
}

#[derive(Component)]
pub(in crate::app::games::mahjong) struct MahjongFinalRow {
    pub(super) opened_at: f32,
    pub(super) index: usize,
}

#[derive(Component)]
pub(in crate::app::games::mahjong) struct MahjongFinalActions {
    pub(super) opened_at: f32,
    pub(super) row_count: usize,
}

pub(in crate::app::games::mahjong) fn animate_mahjong_final_summary(
    time: Res<Time>,
    mut rows: Query<
        (
            &MahjongFinalRow,
            &mut Visibility,
            &mut UiTransform,
            &mut ImageNode,
        ),
        Without<MahjongFinalActions>,
    >,
    mut actions: Query<(&MahjongFinalActions, &mut Visibility), Without<MahjongFinalRow>>,
) {
    let now = time.elapsed_secs();
    for (row, mut visible, mut transform, mut image) in &mut rows {
        let progress = summary_row_progress(
            now - row.opened_at,
            SUMMARY_ROW_START_DELAY + row.index as f32 * SUMMARY_ROW_INTERVAL,
        );
        *visible = if progress > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation = Val2::px(0.0, 12.0 * (1.0 - progress));
        image.color = Color::WHITE.with_alpha(progress);
    }
    for (state, mut visible) in &mut actions {
        let delay = SUMMARY_ROW_START_DELAY
            + state.row_count as f32 * SUMMARY_ROW_INTERVAL
            + SUMMARY_ACTIONS_EXTRA_DELAY;
        *visible = if now - state.opened_at >= delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[expect(
    clippy::type_complexity,
    reason = "ParamSet keeps simultaneous page and score-stage animation queries disjoint"
)]
pub(in crate::app::games::mahjong) fn animate_mahjong_settlement(
    animation: Res<GameSummaryAnimation>,
    mut visibility: ParamSet<(
        Query<(&MahjongFanPage, &mut Visibility)>,
        Query<(&MahjongFanEntry, &mut Visibility, &mut UiTransform)>,
        Query<(&MahjongScoreStage, &mut Visibility)>,
        Query<(
            &MahjongFlyingDelta,
            &mut Visibility,
            &mut UiTransform,
            &mut TextColor,
        )>,
    )>,
    mut backgrounds: ParamSet<(
        Query<(&MahjongFanPageCurtain, &mut BackgroundColor)>,
        Query<(&MahjongScoreShade, &mut BackgroundColor)>,
    )>,
    mut scores: Query<(&MahjongScoreValue, &mut Text)>,
) {
    if !animation.is_changed() {
        return;
    }
    let elapsed = animation.elapsed;
    for (page, mut visibility) in &mut visibility.p0() {
        *visibility = if elapsed >= page.start && elapsed < page.end {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (curtain, mut color) in &mut backgrounds.p0() {
        let opacity =
            ((elapsed - (curtain.end - PAGE_FADE_DURATION)) / PAGE_FADE_DURATION).clamp(0.0, 1.0);
        color.0 = Color::srgb(0.035, 0.039, 0.055).with_alpha(opacity);
    }
    for (entry, mut visibility, mut transform) in &mut visibility.p1() {
        let progress = summary_row_progress(elapsed, entry.delay);
        *visibility = if progress > 0.0 && elapsed < entry.end {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        transform.translation = Val2::px(0.0, 12.0 * (1.0 - progress));
    }
    for (stage, mut visibility) in &mut visibility.p2() {
        *visibility = if elapsed >= stage.start && elapsed < stage.end {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (shade, mut color) in &mut backgrounds.p1() {
        let alpha = ((elapsed - shade.start) / SCORE_FADE_DURATION).clamp(0.0, 1.0);
        color.0 = Color::BLACK.with_alpha(0.80 * alpha);
    }
    for (score, mut text) in &mut scores {
        text.0 = score_at(elapsed, score.from, score.to, score.start).to_string();
    }
    for (delta, mut visible, mut transform, mut color) in &mut visibility.p3() {
        let progress = delta_progress(elapsed, delta.start);
        *visible = if elapsed >= delta.appear_at && progress < 1.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        *transform = delta_transform(progress, delta.origin, delta.rotation);
        color.0 = delta.color.with_alpha(1.0 - progress.powi(3));
    }
}

pub(crate) fn mahjong_summary_descriptor(game: &MahjongSnapshot) -> Option<SummaryDescriptor> {
    let MahjongPhaseView::Finished { result } = &game.phase else {
        return None;
    };
    let continue_delay = SettlementTimeline::new(result).continue_at;
    let entry_count =
        ((continue_delay + 0.05 - SUMMARY_ROW_START_DELAY - SUMMARY_ACTIONS_EXTRA_DELAY)
            / SUMMARY_ROW_INTERVAL)
            .ceil() as usize;
    Some(SummaryDescriptor {
        match_id: game.match_id,
        texas_hand_number: None,
        settlement_index: Some(u32::from(result.sequence_index)),
        entry_count: entry_count.max(game.players.len()),
        nonnegative_outcome: if result.match_complete {
            result
                .reference_changes
                .iter()
                .find(|change| change.player == game.you)
                .is_none_or(|change| change.delta >= 0)
        } else {
            result.deltas[usize::from(game.you.0)] >= 0
        },
        reveal_duration: if result.winners.is_empty() {
            0.0
        } else {
            mahjong_win_reveal_duration(result)
        },
    })
}

pub(crate) fn mahjong_summary_pause(game: &MahjongSnapshot, ui: &MahjongUiState) -> Option<f32> {
    let MahjongPhaseView::Finished { result } = &game.phase else {
        return None;
    };
    if ui.fan_summary_continued == Some((game.match_id, result.sequence_index)) {
        return None;
    }
    mahjong_fan_pause_at(result)
}
