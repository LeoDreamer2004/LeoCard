use super::super::{TexasUiAction, TexasUiState};
use super::{TexasRaiseInput, TexasRaiseSlider, raise_from_fraction};
use crate::app::shell::{PressedUiAction, UiAction, UiState};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::RelativeCursorPosition;
use leocard_texas_holdem::TexasRaiseSizing;
use leocard_ui::{TextInputAction, TextInputEvent};

pub(in super::super) fn focus_texas_raise_input(
    inputs: Query<Entity, Added<TexasRaiseInput>>,
    mut focus: ResMut<InputFocus>,
    mut game_ui: ResMut<TexasUiState>,
) {
    for editor in &inputs {
        focus.set(editor, FocusCause::Auto);
        game_ui.raise_input_valid = true;
    }
}

pub(in super::super) fn handle_texas_raise_slider(
    mouse: Res<ButtonInput<MouseButton>>,
    sliders: Query<(
        Entity,
        &PickingInteraction,
        &RelativeCursorPosition,
        &TexasRaiseSlider,
    )>,
    mut dragging: Local<Option<Entity>>,
    mut game_ui: ResMut<TexasUiState>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = sliders.iter().find_map(|(entity, interaction, _, _)| {
            (*interaction == PickingInteraction::Pressed).then_some(entity)
        });
    }
    if !mouse.pressed(MouseButton::Left) {
        *dragging = None;
        return;
    }
    if let Some(entity) = *dragging
        && let Ok((_, _, cursor, slider)) = sliders.get(entity)
        && let Some(position) = cursor.normalized
    {
        let target = raise_from_fraction(slider.0, position.x + 0.5);
        if target != game_ui.raise_to {
            game_ui.raise_to = target;
            game_ui.raise_input_valid = true;
        }
    }
}

pub(in super::super) fn sync_texas_raise_input(
    mut inputs: Query<(Entity, &TexasRaiseInput, &mut EditableText)>,
    mut events: MessageReader<TextInputEvent>,
    mut actions: MessageWriter<PressedUiAction>,
    mut focus: ResMut<InputFocus>,
    mut game_ui: ResMut<TexasUiState>,
    mut ui: ResMut<UiState>,
) {
    for (entity, sizing, input) in &mut inputs {
        if focus.get() == Some(entity) && input.is_changed() {
            let target = parse_raise(&input, sizing.0);
            game_ui.raise_input_valid = target.is_some();
            if let Some(target) = target
                && target != game_ui.raise_to
            {
                game_ui.raise_to = target;
            }
        }
    }
    for event in events.read() {
        if event.action == TextInputAction::Submit && focus.get() != Some(event.entity) {
            continue;
        }
        let Ok((_, sizing, input)) = inputs.get(event.entity) else {
            continue;
        };
        if event.action == TextInputAction::Submit {
            let Some(target) = parse_raise(input, sizing.0) else {
                continue;
            };
            game_ui.raise_to = target;
            if !game_ui.raise_input_valid {
                game_ui.raise_input_valid = true;
            }
            actions.write(PressedUiAction(UiAction::Texas(TexasUiAction::Raise)));
            ui.dirty = true;
        } else {
            actions.write(PressedUiAction(UiAction::Texas(TexasUiAction::CancelRaise)));
            ui.dirty = true;
        }
        focus.clear();
    }
}

pub(in super::super) fn sync_texas_raise_value(
    mut state: ResMut<TexasUiState>,
    focus: Res<InputFocus>,
    mut inputs: Query<(Entity, &mut EditableText), With<TexasRaiseInput>>,
) {
    let value = state.raise_to.to_string();
    for (entity, mut input) in &mut inputs {
        if focus.get() != Some(entity) {
            if !state.raise_input_valid {
                state.raise_input_valid = true;
            }
            if input.value() != value.as_str() {
                input.editor_mut().set_text(&value);
            }
        }
    }
}

fn parse_raise(input: &EditableText, sizing: TexasRaiseSizing) -> Option<u32> {
    sizing.normalize(input.value().to_string().parse().ok()?)
}
