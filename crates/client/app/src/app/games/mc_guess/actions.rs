use super::graph::{GraphGuideState, GraphLog};
use super::state::{GuessInput, McGuessUiState};
use crate::app::runtime::ClientResource;
use crate::app::shell::{DomainUiAction, PressedUiAction, UiAction, game_command};
use bevy::{prelude::*, text::EditableText};
use leocard_mc_guess::McGuessRuleSet;
use leocard_protocol::{ClientCommand, GameSnapshot, McGuessCommand};

use leocard_ui::{TextInputAction, TextInputEvent};

#[derive(Clone)]
pub(crate) enum McGuessUiAction {
    UpdateRules(McGuessRuleSet),
    ToggleModeMenu,
    CloseModeMenu,
    ToggleGraphGuide,
    CloseGraphGuide,
    ToggleGraphLog,
    Guess(Option<String>),
    Skip,
    Again,
    Lobby,
    DeveloperActivateAll,
}

impl DomainUiAction for McGuessUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::McGuess(action) = action else {
            return None;
        };
        Some(action)
    }

    fn rebuilds_ui(&self) -> bool {
        matches!(
            self,
            Self::UpdateRules(_) | Self::ToggleModeMenu | Self::CloseModeMenu
        )
    }
}

pub(super) fn dispatch_guess_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut client: Option<ResMut<ClientResource>>,
    mut state: ResMut<McGuessUiState>,
    mut guide: ResMut<GraphGuideState>,
    mut log: ResMut<GraphLog>,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    for pressed in actions.read() {
        let Some(action) = McGuessUiAction::extract(&pressed.0) else {
            continue;
        };
        let command = match action {
            McGuessUiAction::ToggleGraphLog => {
                log.toggle();
                continue;
            }
            McGuessUiAction::ToggleGraphGuide => {
                guide.toggle();
                continue;
            }
            McGuessUiAction::CloseGraphGuide => {
                guide.close();
                continue;
            }
            McGuessUiAction::ToggleModeMenu => {
                state.mode_menu_open = !state.mode_menu_open;
                continue;
            }
            McGuessUiAction::CloseModeMenu => {
                state.mode_menu_open = false;
                continue;
            }
            McGuessUiAction::UpdateRules(rules) => {
                state.mode_menu_open = false;
                game_command(McGuessCommand::UpdateRules { rules: *rules })
            }
            McGuessUiAction::Guess(entry) => {
                if !client
                    .0
                    .model()
                    .game_snapshot()
                    .and_then(GameSnapshot::mc_guess)
                    .is_some_and(|game| state.can_guess(game))
                {
                    continue;
                }
                game_command(McGuessCommand::Guess {
                    entry: entry.as_ref().unwrap_or(&state.input).clone(),
                })
            }
            McGuessUiAction::Skip => game_command(McGuessCommand::Skip),
            McGuessUiAction::Again => ClientCommand::PlayAgain,
            McGuessUiAction::Lobby => ClientCommand::ReturnToLobby,
            McGuessUiAction::DeveloperActivateAll => {
                if !(cfg!(debug_assertions) || cfg!(feature = "developer")) {
                    continue;
                }
                game_command(McGuessCommand::DeveloperActivateAll)
            }
        };
        client.0.send(command);
    }
}

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
        if !client.as_deref().is_some_and(|client| {
            client
                .0
                .model()
                .lobby()
                .is_some_and(|lobby| lobby.rules.mc_guess().is_some())
        }) {
            state.mode_menu_open = false;
        }
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
