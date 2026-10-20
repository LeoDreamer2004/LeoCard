use super::super::MinecraftGuessUiState;
use super::{super::state::GuessInput, candidates::GuessCandidate};
use crate::app::{presentation::UiPress, runtime::ClientResource};
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    input_focus::{FocusCause, FocusedInput, InputFocus},
    prelude::*,
    text::EditableText,
};

pub(in super::super) fn navigate_guess_candidates(
    mut event: On<FocusedInput<KeyboardInput>>,
    inputs: Query<(Entity, &EditableText), With<GuessInput>>,
    candidates: Query<(Entity, &GuessCandidate)>,
    client: Option<Res<ClientResource>>,
    state: Res<MinecraftGuessUiState>,
    mut focus: ResMut<InputFocus>,
    mut presses: MessageWriter<UiPress>,
) {
    let origin = event.original_event_target();
    if event.focused_entity != origin || event.input.state != ButtonState::Pressed {
        return;
    }
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().minecraft_guess_game())
    else {
        return;
    };
    if !game.rules.options().candidate_hints || !state.can_guess(game) {
        return;
    }
    let Ok((editor, input)) = inputs.single() else {
        return;
    };
    if input.is_composing() {
        return;
    }
    let mut options: Vec<_> = candidates.iter().collect();
    options.sort_unstable_by_key(|(_, candidate)| candidate.0);
    let key = event.input.key_code;
    let target = if origin == editor && key == KeyCode::ArrowDown {
        options.first().map(|(entity, _)| *entity)
    } else if let Some(index) = options.iter().position(|(entity, _)| *entity == origin) {
        match key {
            KeyCode::ArrowLeft => Some(options[(index + options.len() - 1) % options.len()].0),
            KeyCode::ArrowRight => Some(options[(index + 1) % options.len()].0),
            KeyCode::ArrowUp | KeyCode::Escape => Some(editor),
            KeyCode::Enter if !event.input.repeat => {
                presses.write(UiPress(origin));
                Some(editor)
            }
            _ => None,
        }
    } else {
        None
    };
    if let Some(target) = target {
        focus.set(target, FocusCause::Navigated);
        event.propagate(false);
    }
}
