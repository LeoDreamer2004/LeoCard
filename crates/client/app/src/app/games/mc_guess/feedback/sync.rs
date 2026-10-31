use super::super::McGuessUiState;
use super::state::{ANSWER_DURATION, FeedbackSound, GuessFeedback, RowReveal};
use crate::app::presentation::StartGameSeatTransition;
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_mc_guess::McGuessResult;
use leocard_protocol::GameSnapshot;

pub(in super::super) fn sync_feedback(
    time: Res<Time>,
    client: Option<Res<ClientResource>>,
    intro: Res<StartGameSeatTransition>,
    mut playback: ResMut<GuessFeedback>,
    state: Res<McGuessUiState>,
    mut sounds: MessageWriter<FeedbackSound>,
) {
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::mc_guess)
    }) else {
        if playback.key.is_some() {
            *playback = GuessFeedback::default();
        }
        return;
    };
    let key = (game.match_id, game.progress.game_index);
    let round = state.viewed_round(game);
    if playback.key != Some(key) {
        let continuing = playback.key.is_some_and(|(id, _)| id == game.match_id);
        *playback = GuessFeedback {
            key: Some(key),
            mode: game.rules.mode,
            graph_guesses: game.graph.as_ref().map_or(0, |graph| graph.guesses.len()),
            // Joining an existing game does not replay its old records or victory.
            rows: if continuing {
                Vec::new()
            } else {
                (0..round.guesses.len()).map(|_| None).collect()
            },
            finished_at: (!continuing && game.round.finished()).then_some(-ANSWER_DURATION),
            skipped: game.round.skipped_players.clone(),
            ..default()
        };
    }
    let selected = state.viewed_node(game);
    if playback.selected_node != selected {
        playback.selected_node = selected;
        playback.rows = (0..round.guesses.len()).map(|_| None).collect();
        playback.cues.clear();
    }
    if !intro.is_active_for(game.match_id) {
        playback.clock += time.delta_secs();
    }
    if let Some(graph) = &game.graph {
        if graph.guesses.len() > playback.graph_guesses
            && !game.round.finished()
            && let Some(record) = graph
                .guesses
                .last()
                .filter(|record| record.guess.result.is_correct())
        {
            sounds.write(FeedbackSound::Combo(record.combo));
        }
        playback.graph_guesses = graph.guesses.len();
    }
    for guess in round.guesses.iter().skip(playback.rows.len()) {
        let columns = match &guess.result {
            McGuessResult::Known { matches, .. } => matches.len() + 1,
            McGuessResult::Unknown { .. } => 2,
        };
        let start = playback.row_start();
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
        if playback.last_player == Some(game.you) {
            let at = playback.tail() + 0.08;
            playback.cue(at, FeedbackSound::Turn);
        }
    }
    if game.round.finished() && playback.finished_at.is_none() {
        let at = playback.tail() + 0.08;
        playback.finished_at = Some(at);
        if game.round.winner.is_some() || game.graph.is_some() {
            playback.cue(at, FeedbackSound::Success);
        }
    }
}
