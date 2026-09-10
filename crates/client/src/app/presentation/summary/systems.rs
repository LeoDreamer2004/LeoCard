//! 跨游戏结算界面的入场、计分与音效演出。

use super::{
    AnimatedSignedSummaryScore, AnimatedSummaryScore, AnimatedSummaryText, GameSummaryActions,
    GameSummaryAnimation, GameSummaryDivider, GameSummaryModal, GameSummaryPanelTexture,
    GameSummaryRow, SUMMARY_ACTIONS_EXTRA_DELAY, SUMMARY_MODAL_ENTRY_DURATION,
    SUMMARY_ROW_ENTRY_DURATION, SUMMARY_ROW_INTERVAL, SUMMARY_ROW_START_DELAY,
    SUMMARY_SCORE_COUNT_DURATION,
};
use crate::app::games::game_summary_descriptor;
use crate::app::{ClientResource, PANEL_ALT, UiAssets, ease_out_cubic};
use bevy::prelude::*;
use leocard_protocol::PlayerScore;

#[expect(
    clippy::too_many_arguments,
    reason = "the text builder keeps animation timing and typography inputs explicit"
)]
pub(crate) fn add_animated_summary_text(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
    size: f32,
    color: Color,
    delay: f32,
    elapsed: f32,
    assets: &UiAssets,
) -> Entity {
    let opacity = if delay == 0.0 {
        summary_modal_visual(elapsed).opacity
    } else {
        summary_row_progress(elapsed, delay)
    };
    let entity = crate::app::add_text(
        commands,
        parent,
        text,
        size,
        color.with_alpha(opacity),
        assets,
    );
    commands
        .entity(entity)
        .insert(AnimatedSummaryText { color, delay });
    entity
}

pub(crate) fn update_summary_animation(
    mut commands: Commands,
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut animation: ResMut<GameSummaryAnimation>,
) {
    let summary = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .and_then(game_summary_descriptor);
    let Some(summary) = summary else {
        if animation.match_id.is_some() {
            *animation = GameSummaryAnimation::default();
        }
        return;
    };

    if animation.match_id != Some(summary.match_id)
        || animation.texas_hand_number != summary.texas_hand_number
        || animation.settlement_index != summary.settlement_index
    {
        animation.match_id = Some(summary.match_id);
        animation.texas_hand_number = summary.texas_hand_number;
        animation.settlement_index = summary.settlement_index;
        animation.entry_count = summary.entry_count;
        animation.elapsed = -summary.reveal_duration;
        animation.nonnegative_outcome = summary.nonnegative_outcome;
        animation.outcome_sound_played = false;
    } else {
        let duration = summary_animation_duration(animation.entry_count);
        if animation.elapsed < duration {
            animation.elapsed = (animation.elapsed + time.delta_secs()).min(duration);
        }
    }

    if animation.elapsed >= 0.0 && !animation.outcome_sound_played {
        let sound = if animation.nonnegative_outcome {
            assets.audio.summary_score_sound.clone()
        } else {
            assets.audio.summary_die_sound.clone()
        };
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
        animation.outcome_sound_played = true;
    }
}

fn summary_animation_duration(player_count: usize) -> f32 {
    let last_row = SUMMARY_ROW_START_DELAY
        + player_count.saturating_sub(1) as f32 * SUMMARY_ROW_INTERVAL
        + SUMMARY_SCORE_COUNT_DURATION.max(SUMMARY_ROW_ENTRY_DURATION);
    let actions = SUMMARY_ROW_START_DELAY
        + player_count as f32 * SUMMARY_ROW_INTERVAL
        + SUMMARY_ACTIONS_EXTRA_DELAY;
    SUMMARY_MODAL_ENTRY_DURATION.max(last_row).max(actions)
}

pub(crate) fn sorted_summary_scores(scores: &[PlayerScore]) -> Vec<PlayerScore> {
    let mut ranked = scores.to_vec();
    ranked.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.player.0.cmp(&right.player.0))
    });
    ranked
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SummaryModalVisual {
    pub offset_y: f32,
    pub opacity: f32,
}

pub(crate) fn summary_modal_visual(elapsed: f32) -> SummaryModalVisual {
    let progress = ease_out_cubic((elapsed / SUMMARY_MODAL_ENTRY_DURATION).clamp(0.0, 1.0));
    SummaryModalVisual {
        offset_y: -52.0 * (1.0 - progress),
        opacity: progress,
    }
}

pub(crate) fn summary_row_progress(elapsed: f32, delay: f32) -> f32 {
    ease_out_cubic(((elapsed - delay) / SUMMARY_ROW_ENTRY_DURATION).clamp(0.0, 1.0))
}

#[expect(
    clippy::type_complexity,
    reason = "the ParamSet keeps overlapping Bevy summary queries disjoint"
)]
pub(crate) fn animate_game_summary_visuals(
    animation: Res<GameSummaryAnimation>,
    mut panels: ParamSet<(
        Query<(&mut UiTransform, &mut Visibility), With<GameSummaryModal>>,
        Query<
            (
                &GameSummaryRow,
                &mut UiTransform,
                &mut BackgroundColor,
                &mut Visibility,
            ),
            Without<GameSummaryModal>,
        >,
        Query<(&GameSummaryActions, &mut Visibility)>,
        Query<(&GameSummaryDivider, &mut BackgroundColor, &mut Visibility)>,
    )>,
    mut texts: Query<(&AnimatedSummaryText, &mut TextColor)>,
    mut panel_textures: Query<&mut ImageNode, With<GameSummaryPanelTexture>>,
) {
    if !animation.is_changed() {
        return;
    }
    let modal = summary_modal_visual(animation.elapsed);
    for mut image in &mut panel_textures {
        image.color = Color::WHITE.with_alpha(0.98 * modal.opacity);
    }
    for (mut transform, mut visibility) in &mut panels.p0() {
        transform.translation = Val2::px(0.0, modal.offset_y);
        *visibility = if animation.elapsed >= 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (row, mut transform, mut background, mut visibility) in &mut panels.p1() {
        let progress = summary_row_progress(animation.elapsed, row.delay);
        transform.translation = Val2::px(0.0, 12.0 * (1.0 - progress));
        background.0 = PANEL_ALT.with_alpha(0.82 * progress);
        *visibility = if progress > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (animated, mut color) in &mut texts {
        let opacity = if animated.delay == 0.0 {
            modal.opacity
        } else {
            summary_row_progress(animation.elapsed, animated.delay)
        };
        color.0 = animated.color.with_alpha(opacity);
    }

    for (animated, mut visibility) in &mut panels.p2() {
        *visibility = if animation.elapsed >= animated.delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (divider, mut background, mut visibility) in &mut panels.p3() {
        let progress = summary_row_progress(animation.elapsed, divider.delay);
        background.0 = Color::srgb(0.52, 0.55, 0.54).with_alpha(0.28 * progress);
        *visibility = if progress > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(crate) fn animated_summary_score(elapsed: f32, target: u32, delay: f32) -> u32 {
    let progress = ((elapsed - delay) / SUMMARY_SCORE_COUNT_DURATION).clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - progress).powi(3);
    (target as f32 * eased).round() as u32
}

pub(crate) fn animate_summary_scores(
    animation: Res<GameSummaryAnimation>,
    mut scores: Query<(&AnimatedSummaryScore, &mut Text)>,
) {
    if !animation.is_changed() {
        return;
    }
    for (score, mut text) in &mut scores {
        let displayed = animated_summary_score(animation.elapsed, score.target, score.delay);
        let expected = format!("{displayed} 分");
        if text.0 != expected {
            text.0 = expected;
        }
    }
}

pub(crate) fn animate_signed_summary_scores(
    animation: Res<GameSummaryAnimation>,
    mut scores: Query<(&AnimatedSignedSummaryScore, &mut Text)>,
) {
    if !animation.is_changed() {
        return;
    }
    for (score, mut text) in &mut scores {
        let progress =
            ((animation.elapsed - score.delay) / SUMMARY_SCORE_COUNT_DURATION).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        let displayed = (score.target as f32 * eased).round() as i32;
        let expected = format!("累计 {displayed:+}");
        if text.0 != expected {
            text.0 = expected;
        }
    }
}
