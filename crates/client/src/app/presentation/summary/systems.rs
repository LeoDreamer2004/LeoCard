//! 跨游戏结算界面的入场、计分与音效演出。

use super::*;
use leocard_protocol::{
    GamePhaseView, MahjongPhaseView, PlayerScore, TexasHoldemPhaseView, UnoPhaseView,
};

pub fn update_summary_animation(
    mut commands: Commands,
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut animation: ResMut<GameSummaryAnimation>,
) {
    let summary = client.as_deref().and_then(|client| {
        if let Some(game) = client.0.model().qigui523_game()
            && let GamePhaseView::Finished {
                match_id,
                scores,
                reference_changes,
                ..
            } = &game.phase
        {
            return Some((
                *match_id,
                None,
                None,
                scores.len(),
                reference_changes
                    .iter()
                    .find(|change| change.player == game.you)
                    .is_none_or(|change| change.delta >= 0),
                SUMMARY_HAND_REVEAL_DURATION,
            ));
        }
        if let Some(game) = client.0.model().uno_game()
            && let UnoPhaseView::Finished {
                results,
                reference_changes,
                ..
            } = &game.phase
        {
            return Some((
                game.match_id,
                None,
                None,
                results.len(),
                reference_changes
                    .iter()
                    .find(|change| change.player == game.you)
                    .is_none_or(|change| change.delta >= 0),
                UNO_FINISH_REVEAL_DURATION,
            ));
        }
        if let Some(game) = client.0.model().mahjong_game()
            && let MahjongPhaseView::Finished { result } = &game.phase
        {
            let fan_entries = result
                .winners
                .iter()
                .map(|winner| winner.score.fans.len() + 1)
                .sum::<usize>();
            return Some((
                game.match_id,
                None,
                Some(u32::from(result.sequence_index)),
                game.players.len() + fan_entries,
                result.deltas[game.you.0 as usize] >= 0,
                if result.winners.is_empty() {
                    0.0
                } else {
                    mahjong_win_reveal_duration(result)
                },
            ));
        }
        let game = client.0.model().texas_holdem_game()?;
        let TexasHoldemPhaseView::HandComplete {
            showdown,
            tournament_complete,
            reference_changes,
            ..
        } = &game.phase
        else {
            return None;
        };
        let own = game.players.iter().find(|player| player.id == game.you)?;
        let nonnegative = if *tournament_complete {
            reference_changes
                .iter()
                .find(|change| change.player == game.you)
                .is_none_or(|change| change.delta >= 0)
        } else {
            own.stack >= own.hand_start_stack
        };
        let mut rows = game
            .players
            .iter()
            .map(|player| (player.id, player.stack))
            .collect::<Vec<_>>();
        if *tournament_complete {
            rows.extend(game.players.iter().map(|player| (player.id, player.stack)));
        }
        Some((
            game.match_id,
            Some(game.hand_number),
            None,
            rows.len(),
            nonnegative,
            if *showdown {
                TEXAS_SHOWDOWN_REVEAL_DURATION
            } else {
                TEXAS_UNCONTESTED_REVEAL_DURATION
            },
        ))
    });
    let Some((
        match_id,
        texas_hand_number,
        settlement_index,
        entry_count,
        nonnegative_outcome,
        reveal_duration,
    )) = summary
    else {
        if animation.match_id.is_some() {
            *animation = GameSummaryAnimation::default();
        }
        return;
    };

    if animation.match_id != Some(match_id)
        || animation.texas_hand_number != texas_hand_number
        || animation.settlement_index != settlement_index
    {
        animation.match_id = Some(match_id);
        animation.texas_hand_number = texas_hand_number;
        animation.settlement_index = settlement_index;
        animation.entry_count = entry_count;
        animation.elapsed = -reveal_duration;
        animation.nonnegative_outcome = nonnegative_outcome;
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

pub fn sorted_summary_scores(scores: &[PlayerScore]) -> Vec<PlayerScore> {
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
pub struct SummaryModalVisual {
    pub offset_y: f32,
    pub opacity: f32,
}

pub fn summary_modal_visual(elapsed: f32) -> SummaryModalVisual {
    let progress = ease_out_cubic((elapsed / SUMMARY_MODAL_ENTRY_DURATION).clamp(0.0, 1.0));
    SummaryModalVisual {
        offset_y: -52.0 * (1.0 - progress),
        opacity: progress,
    }
}

pub fn summary_row_progress(elapsed: f32, delay: f32) -> f32 {
    ease_out_cubic(((elapsed - delay) / SUMMARY_ROW_ENTRY_DURATION).clamp(0.0, 1.0))
}

pub fn animate_game_summary_visuals(
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

pub fn animated_summary_score(elapsed: f32, target: u32, delay: f32) -> u32 {
    let progress = ((elapsed - delay) / SUMMARY_SCORE_COUNT_DURATION).clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - progress).powi(3);
    (target as f32 * eased).round() as u32
}

pub fn animate_summary_scores(
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

pub fn animate_signed_summary_scores(
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
