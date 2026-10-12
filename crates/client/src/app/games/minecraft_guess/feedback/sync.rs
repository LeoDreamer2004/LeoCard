use super::state::{ANSWER_DURATION, FeedbackSound, GuessFeedback, RowReveal};
use crate::app::presentation::StartGameSeatTransition;
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_minecraft_guess::GuessResult;

pub(in super::super) fn sync_feedback(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    intro: Res<StartGameSeatTransition>,
    mut playback: ResMut<GuessFeedback>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().minecraft_guess_game())
    else {
        if playback.key.is_some() {
            *playback = GuessFeedback::default();
        }
        return;
    };
    let key = (game.match_id, game.progress.game_index);
    if playback.key != Some(key) {
        let continuing = playback.key.is_some_and(|(id, _)| id == game.match_id);
        *playback = GuessFeedback {
            key: Some(key),
            // Joining an existing game does not replay its old records or victory.
            rows: if continuing {
                Vec::new()
            } else {
                (0..game.round.guesses.len()).map(|_| None).collect()
            },
            finished_at: (!continuing && game.round.finished()).then_some(-ANSWER_DURATION),
            skipped: game.round.skipped_players.clone(),
            ..default()
        };
    }
    if !intro.is_active_for(game.match_id) {
        playback.clock += time.delta_secs();
    }
    for guess in game.round.guesses.iter().skip(playback.rows.len()) {
        let columns = match &guess.result {
            GuessResult::Known { matches, .. } => matches.len() + 1,
            GuessResult::Unknown { .. } => 2,
        };
        let start = playback.tail();
        playback.rows.push(Some(RowReveal { start, columns }));
        playback.cue(start, FeedbackSound::Submit);
        playback.cue(start + 0.10, FeedbackSound::Reveal);
    }
    if game.round.skipped_players != playback.skipped {
        playback.skipped.clone_from(&game.round.skipped_players);
        let at = playback.tail();
        playback.cue(at, FeedbackSound::Skip);
    }
    if game.round.current_player != playback.last_player {
        playback.last_player = game.round.current_player;
        if playback.last_player == Some(game.you.0) {
            let at = playback.tail() + 0.08;
            playback.cue(at, FeedbackSound::Turn);
        }
    }
    if game.round.finished() && playback.finished_at.is_none() {
        let at = playback.tail() + 0.08;
        playback.finished_at = Some(at);
        if game.round.winner.is_some() {
            playback.cue(at, FeedbackSound::Success);
        }
    }
}
