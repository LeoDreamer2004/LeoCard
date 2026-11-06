use super::state::{GuessInput, McGuessUiState};
use crate::app::runtime::ClientResource;
use crate::app::shell::game_command;
use bevy::prelude::*;
use bevy::text::EditableText;
use leocard_protocol::{GameSnapshot, McGuessCommand};
use leocard_ui::{TextInputAction, TextInputEvent};

pub(super) fn sync_guess_input(
    mut events: MessageReader<TextInputEvent>,
    mut inputs: Query<(Entity, &mut EditableText), With<GuessInput>>,
    mut state: ResMut<McGuessUiState>,
    mut client: Option<ResMut<ClientResource>>,
) {
    let snapshot = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::mc_guess)
    });
    let Some(snapshot) = snapshot else {
        return;
    };
    if state.match_id != Some(snapshot.match_id) || state.game_index != snapshot.progress.game_index
    {
        *state = McGuessUiState {
            match_id: Some(snapshot.match_id),
            game_index: snapshot.progress.game_index,
            ..default()
        };
        for (_, mut input) in &mut inputs {
            input.clear();
        }
    }
    let selected = state.viewed_node(snapshot);
    let round = state.viewed_round(snapshot);
    if state.selected_node != selected {
        state.selected_node = selected;
        state.guess_count = round.guesses.len();
        state.scroll_y = 0.0;
        state.input.clear();
        for (_, mut input) in &mut inputs {
            input.clear();
        }
    }
    if state.guess_count != round.guesses.len() {
        let previous_count = state.guess_count;
        state.guess_count = round.guesses.len();
        state.scroll_to_latest = true;
        if round
            .guesses
            .iter()
            .skip(previous_count)
            .any(|guess| guess.player == snapshot.you)
        {
            state.input.clear();
            for (_, mut input) in &mut inputs {
                input.clear();
            }
        }
    }
    let can_guess = state.can_guess(snapshot);
    drop(round);
    for event in events.read() {
        if event.action == TextInputAction::Submit
            && can_guess
            && let Ok((_, input)) = inputs.get(event.entity)
            && !input.value().to_string().trim().is_empty()
            && let Some(client) = client.as_deref_mut()
        {
            client.0.send(game_command(McGuessCommand::Guess {
                entry: input.value().to_string(),
            }));
        }
    }
    for (_, input) in &inputs {
        let value = input.value().to_string();
        if state.input != value {
            state.input = value;
        }
    }
}
