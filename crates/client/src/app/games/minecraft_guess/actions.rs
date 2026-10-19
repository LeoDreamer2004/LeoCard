use super::state::{GuessInput, MinecraftGuessUiState};
use crate::app::{
    presentation::{TextInputAction, TextInputEvent},
    runtime::ClientResource,
    shell::{DomainUiAction, PressedUiAction, UiAction, game_command},
};
use bevy::{prelude::*, text::EditableText};
use leocard_minecraft_guess::MinecraftGuessRuleSet;
use leocard_protocol::{ClientCommand, MinecraftGuessCommand};

#[derive(Clone)]
pub(crate) enum MinecraftGuessUiAction {
    UpdateRules(MinecraftGuessRuleSet),
    ToggleModeMenu,
    CloseModeMenu,
    Guess(Option<String>),
    Skip,
    Again,
    Lobby,
}

impl DomainUiAction for MinecraftGuessUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::MinecraftGuess(action) = action else {
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
    mut state: ResMut<MinecraftGuessUiState>,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    for pressed in actions.read() {
        let Some(action) = MinecraftGuessUiAction::extract(&pressed.0) else {
            continue;
        };
        let command = match action {
            MinecraftGuessUiAction::ToggleModeMenu => {
                state.mode_menu_open = !state.mode_menu_open;
                continue;
            }
            MinecraftGuessUiAction::CloseModeMenu => {
                state.mode_menu_open = false;
                continue;
            }
            MinecraftGuessUiAction::UpdateRules(rules) => {
                state.mode_menu_open = false;
                game_command(MinecraftGuessCommand::UpdateRules { rules: *rules })
            }
            MinecraftGuessUiAction::Guess(entry) => game_command(MinecraftGuessCommand::Guess {
                entry: entry.as_ref().unwrap_or(&state.input).clone(),
            }),
            MinecraftGuessUiAction::Skip => game_command(MinecraftGuessCommand::Skip),
            MinecraftGuessUiAction::Again => ClientCommand::PlayAgain,
            MinecraftGuessUiAction::Lobby => ClientCommand::ReturnToLobby,
        };
        client.0.send(command);
    }
}

pub(super) fn sync_guess_input(
    mut events: MessageReader<TextInputEvent>,
    mut inputs: Query<(Entity, &mut EditableText), With<GuessInput>>,
    mut state: ResMut<MinecraftGuessUiState>,
    mut client: Option<ResMut<ClientResource>>,
) {
    let snapshot = client
        .as_deref()
        .and_then(|client| client.0.model().minecraft_guess_game());
    let Some(snapshot) = snapshot else {
        if !client.as_deref().is_some_and(|client| {
            client
                .0
                .model()
                .lobby()
                .is_some_and(|lobby| lobby.rules.minecraft_guess().is_some())
        }) {
            state.mode_menu_open = false;
        }
        return;
    };
    if state.match_id != Some(snapshot.match_id) || state.game_index != snapshot.progress.game_index
    {
        *state = MinecraftGuessUiState {
            match_id: Some(snapshot.match_id),
            game_index: snapshot.progress.game_index,
            ..default()
        };
        for (_, mut input) in &mut inputs {
            input.clear();
        }
    }
    let selected = snapshot.graph.as_ref().map(|graph| graph.selected);
    if state.selected_node != selected {
        state.selected_node = selected;
        state.guess_count = snapshot.round.guesses.len();
        state.scroll_y = 0.0;
        state.input.clear();
        for (_, mut input) in &mut inputs {
            input.clear();
        }
    }
    if state.guess_count != snapshot.round.guesses.len() {
        let previous_count = state.guess_count;
        state.guess_count = snapshot.round.guesses.len();
        state.scroll_to_latest = true;
        if snapshot
            .round
            .guesses
            .iter()
            .skip(previous_count)
            .any(|guess| guess.player == snapshot.you.0)
        {
            state.input.clear();
            for (_, mut input) in &mut inputs {
                input.clear();
            }
        }
    }
    let can_guess = snapshot.can_guess();
    for event in events.read() {
        if event.action == TextInputAction::Submit
            && can_guess
            && let Ok((_, input)) = inputs.get(event.entity)
            && !input.value().to_string().trim().is_empty()
            && let Some(client) = client.as_deref_mut()
        {
            client.0.send(game_command(MinecraftGuessCommand::Guess {
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
