use super::{ShengjiHandCardSelectionOverlay, ShengjiHandCardSlot, ShengjiUiState};
use crate::app::presentation::{CardDragSelection, drag_preview_color, update_drag_selection};
use crate::app::shell::UiState;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

pub(crate) fn handle_shengji_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<CardDragSelection>,
    mut game_ui: ResMut<ShengjiUiState>,
    mut ui: ResMut<UiState>,
    cards: Query<(
        &PickingInteraction,
        &RelativeCursorPosition,
        &ShengjiHandCardSlot,
    )>,
) {
    let pressed = cards
        .iter()
        .find(|(interaction, _, _)| **interaction == PickingInteraction::Pressed)
        .map(|(_, _, slot)| (slot.index, slot.card));
    let hovered = cards
        .iter()
        .filter(|(_, cursor, _)| cursor.cursor_over())
        .map(|(_, _, slot)| slot.index)
        .max();
    ui.dirty |= update_drag_selection(
        &mouse,
        &mut drag,
        &mut game_ui.selected,
        pressed,
        hovered,
        cards.iter().map(|(_, _, slot)| (slot.index, slot.card)),
    );
}

pub(crate) fn sync_shengji_card_drag_preview(
    drag: Res<CardDragSelection>,
    mut overlays: Query<(&ShengjiHandCardSelectionOverlay, &mut BackgroundColor)>,
) {
    if !drag.is_changed() {
        return;
    }
    for (overlay, mut color) in &mut overlays {
        let expected = drag_preview_color(&drag, overlay.index);
        if color.0 != expected {
            color.0 = expected;
        }
    }
}
