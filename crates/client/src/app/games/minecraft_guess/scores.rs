use crate::app::{
    presentation::{ACCENT, SCORE_GAIN_DURATION, rolling_score, score_gain_visual},
    runtime::ClientResource,
};
use bevy::prelude::*;
use leocard_minecraft_guess::ScoreGain;
use leocard_protocol::{MatchId, PlayerId};
use std::collections::{HashMap, VecDeque};

#[derive(Resource, Default)]
pub(crate) struct GuessScorePlayback {
    match_id: Option<MatchId>,
    seen: usize,
    scores: HashMap<u8, u32>,
    queue: VecDeque<ScoreGain>,
    active: Option<(ScoreGain, f32)>,
}

impl GuessScorePlayback {
    pub fn score(&self, player: PlayerId, fallback: u32) -> u32 {
        self.active
            .filter(|(gain, _)| gain.player == player.0)
            .map_or_else(
                || self.scores.get(&player.0).copied().unwrap_or(fallback),
                |(gain, elapsed)| rolling_score(gain.before, gain.after, elapsed),
            )
    }

    pub fn gain(&self, player: PlayerId) -> Option<(ScoreGain, f32)> {
        self.active.filter(|(gain, _)| gain.player == player.0)
    }

    pub fn busy(&self) -> bool {
        self.active.is_some() || !self.queue.is_empty()
    }
}

#[derive(Component)]
pub(super) struct GuessScoreValue(pub PlayerId);

#[derive(Component)]
pub(super) struct GuessScoreDelta(pub PlayerId);

pub(super) fn sync_score_playback(
    client: Option<Res<ClientResource>>,
    mut playback: ResMut<GuessScorePlayback>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().minecraft_guess_game())
    else {
        if playback.match_id.is_some() {
            *playback = GuessScorePlayback::default();
        }
        return;
    };
    if playback.match_id != Some(game.match_id) {
        *playback = GuessScorePlayback {
            match_id: Some(game.match_id),
            seen: game.score_gains.len(),
            scores: game
                .scores
                .iter()
                .map(|score| (score.player.0, score.score))
                .collect(),
            ..default()
        };
    } else {
        let seen = playback.seen;
        playback
            .queue
            .extend(game.score_gains[seen..].iter().copied());
        playback.seen = game.score_gains.len();
    }
}

pub(super) fn animate_scores(
    time: Res<Time>,
    mut playback: ResMut<GuessScorePlayback>,
    mut values: Query<(&GuessScoreValue, &mut Text), Without<GuessScoreDelta>>,
    mut deltas: Query<
        (
            &GuessScoreDelta,
            &mut Text,
            &mut TextColor,
            &mut UiTransform,
        ),
        Without<GuessScoreValue>,
    >,
) {
    if let Some((gain, elapsed)) = playback.active.as_mut() {
        *elapsed += time.delta_secs();
        if *elapsed >= SCORE_GAIN_DURATION {
            let (gain, _) = (*gain, *elapsed);
            playback.scores.insert(gain.player, gain.after);
            playback.active = None;
        }
    }
    if playback.active.is_none() {
        playback.active = playback.queue.pop_front().map(|gain| (gain, 0.0));
    }
    for (player, mut text) in &mut values {
        let value = playback.score(player.0, 0).to_string();
        if text.0 != value {
            text.0 = value;
        }
    }
    for (player, mut text, mut color, mut transform) in &mut deltas {
        if let Some((gain, elapsed)) = playback.gain(player.0) {
            text.0 = format!("+{}", gain.after - gain.before);
            let (pose, alpha) = score_gain_visual(elapsed);
            *transform = pose;
            color.0 = ACCENT.with_alpha(alpha);
        } else {
            color.0 = Color::NONE;
        }
    }
}
