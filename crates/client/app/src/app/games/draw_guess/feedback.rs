use super::audio::GuessSound;
use crate::app::presentation::{
    READY, SCORE_GAIN_DURATION, ease_out_cubic, rolling_score, score_gain_visual,
};
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_draw_guess::{DrawGuessPhase, DrawGuessRuleSet};
use leocard_game_common::PlayerId;
use leocard_protocol::{DrawGuessSnapshot, GameSnapshot, MatchId};
use leocard_ui::{TransitionVisuals, UiComposer, fade_panel};
use std::collections::HashMap;
use std::mem;

#[derive(Clone, Copy)]
struct ScoreGain {
    before: u32,
    after: u32,
    elapsed: f32,
}

#[derive(Resource, Default)]
pub(super) struct DrawingFeedback {
    match_id: Option<MatchId>,
    phase: Option<(u32, DrawGuessPhase)>,
    scores: HashMap<PlayerId, u32>,
    gains: HashMap<PlayerId, ScoreGain>,
    messages: HashMap<u32, f32>,
    seen_message: u32,
    entry_elapsed: f32,
    fresh: bool,
    sounds: Vec<GuessSound>,
}

#[derive(Component)]
pub(super) struct DrawingScoreValue(pub PlayerId);

#[derive(Component)]
pub(super) struct DrawingScoreDelta(pub PlayerId);

#[derive(Component)]
pub(super) struct DrawingHistoryMessage {
    pub serial: u32,
}

#[derive(Component)]
pub(super) struct DrawingChoicePanel;

pub(super) fn sync_feedback(
    client: Option<Res<ClientResource>>,
    mut feedback: ResMut<DrawingFeedback>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .and_then(GameSnapshot::draw_guess)
    else {
        if feedback.match_id.is_some() {
            *feedback = DrawingFeedback::default();
        }
        return;
    };
    feedback.observe(game);
}

impl DrawingFeedback {
    pub fn observe(&mut self, game: &DrawGuessSnapshot) {
        let latest = game
            .state
            .history
            .last()
            .map_or(0, |message| message.serial);
        let joining = self.match_id != Some(game.match_id);
        if joining {
            *self = DrawingFeedback {
                match_id: Some(game.match_id),
                seen_message: latest,
                scores: game
                    .state
                    .players
                    .iter()
                    .map(|player| (player.id, player.score))
                    .collect(),
                ..default()
            };
        }
        let reveal = !joining
            && self.phase != Some((game.state.turn, game.state.phase))
            && game.state.phase == DrawGuessPhase::Reveal
            && game.state.word.is_some();
        if self.phase != Some((game.state.turn, game.state.phase)) {
            self.phase = Some((game.state.turn, game.state.phase));
            self.entry_elapsed = if game.state.phase == DrawGuessPhase::Reveal {
                (f32::from(DrawGuessRuleSet::REVEAL_SECONDS)
                    - game.state.remaining_ms as f32 / 1000.0)
                    .max(0.0)
            } else {
                0.0
            };
            self.fresh = true;
        }
        if latest > self.seen_message {
            let seen = self.seen_message;
            for message in game
                .state
                .history
                .iter()
                .filter(|message| message.serial > seen)
            {
                self.messages.insert(message.serial, 0.0);
                if message.correct {
                    self.sounds.push(GuessSound::Correct);
                }
            }
            for player in &game.state.players {
                let before = self.gains.get(&player.id).map_or_else(
                    || self.scores.get(&player.id).copied().unwrap_or(player.score),
                    |gain| rolling_score(gain.before, gain.after, gain.elapsed),
                );
                let old = self
                    .scores
                    .insert(player.id, player.score)
                    .unwrap_or(player.score);
                if player.score > old {
                    self.gains.insert(
                        player.id,
                        ScoreGain {
                            before,
                            after: player.score,
                            elapsed: 0.0,
                        },
                    );
                }
            }
            self.seen_message = latest;
            self.messages.retain(|serial, _| {
                game.state
                    .history
                    .iter()
                    .any(|message| message.serial == *serial)
            });
        }
        if reveal {
            self.sounds.push(GuessSound::Reveal);
        }
    }

    pub fn take_sounds(&mut self) -> Vec<GuessSound> {
        mem::take(&mut self.sounds)
    }

    pub fn answer_elapsed(&self) -> Option<f32> {
        self.phase
            .filter(|(_, phase)| *phase == DrawGuessPhase::Reveal)
            .map(|_| self.entry_elapsed)
    }

    pub fn scores_busy(&self) -> bool {
        !self.gains.is_empty()
    }
}

type ScoreDeltaLabels<'w, 's> = Query<
    'w,
    's,
    (
        &'static DrawingScoreDelta,
        &'static mut Text,
        &'static mut TextColor,
        &'static mut UiTransform,
    ),
    (Without<DrawingScoreValue>, Without<DrawingHistoryMessage>),
>;

pub(super) fn animate_feedback(
    time: Res<Time>,
    mut feedback: ResMut<DrawingFeedback>,
    mut values: Query<(&DrawingScoreValue, &mut Text), Without<DrawingScoreDelta>>,
    mut gains: ScoreDeltaLabels,
) {
    let delta = time.delta_secs();
    if feedback.fresh {
        feedback.fresh = false;
    } else {
        feedback.entry_elapsed += delta;
    }
    feedback.gains.retain(|_, gain| {
        gain.elapsed += delta;
        gain.elapsed < SCORE_GAIN_DURATION
    });
    for elapsed in feedback.messages.values_mut() {
        *elapsed += delta;
    }
    for (player, mut text) in &mut values {
        let value = feedback.gains.get(&player.0).map_or_else(
            || feedback.scores.get(&player.0).copied().unwrap_or(0),
            |gain| rolling_score(gain.before, gain.after, gain.elapsed),
        );
        let next = value.to_string();
        if text.0 != next {
            text.0 = next;
        }
    }
    for (player, mut text, mut color, mut transform) in &mut gains {
        if let Some(gain) = feedback.gains.get(&player.0) {
            let (pose, alpha) = score_gain_visual(gain.elapsed);
            text.0 = format!("+{}", gain.after - gain.before);
            color.0 = READY.with_alpha(alpha);
            *transform = pose;
        } else {
            text.0.clear();
        }
    }
}

pub(super) fn animate_history(
    feedback: Res<DrawingFeedback>,
    mut messages: Query<(Entity, &DrawingHistoryMessage, &mut UiTransform)>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
    mut commands: UiComposer,
) {
    for (entity, message, mut transform) in &mut messages {
        let progress = ease_out_cubic(
            (feedback
                .messages
                .get(&message.serial)
                .copied()
                .unwrap_or(1.0)
                / 0.16)
                .min(1.0),
        );
        transform.translation = Val2::px(-8.0 * (1.0 - progress), 0.0);
        fade_panel(entity, progress, &children, &mut visuals, &mut commands);
    }
}

pub(super) fn animate_choices(
    feedback: Res<DrawingFeedback>,
    mut panels: Query<(Entity, &mut UiTransform), With<DrawingChoicePanel>>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
    mut commands: UiComposer,
) {
    let progress = ease_out_cubic((feedback.entry_elapsed / 0.20).min(1.0));
    for (entity, mut transform) in &mut panels {
        transform.translation = Val2::px(0.0, 12.0 * (1.0 - progress));
        transform.scale = Vec2::splat(0.98 + 0.02 * progress);
        fade_panel(entity, progress, &children, &mut visuals, &mut commands);
    }
}
