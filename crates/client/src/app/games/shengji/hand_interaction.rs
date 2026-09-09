use super::{ShengjiHandCardSelectionOverlay, ShengjiHandCardSlot, ShengjiUiState};
use crate::app::{
    CardDragSelection, CardSize, UiState, advance_towards, drag_preview_color,
    shengji_hand_card_reveal, slot_hover_target, update_drag_selection,
};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

const HAND_CARD_HOVER_WIDTH: f32 = 32.0;

pub(crate) fn animate_shengji_hand_card_slots(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<ShengjiUiState>,
    mut cards: Query<
        (
            &Interaction,
            &RelativeCursorPosition,
            &mut ShengjiHandCardSlot,
            &mut Node,
        ),
        With<Button>,
    >,
) {
    let response = 1.0 - (-18.0 * time.delta_secs()).exp();
    for (interaction, cursor, mut slot, mut node) in &mut cards {
        let target = slot_hover_target(&drag, slot.index, *interaction, cursor);
        slot.hover_amount = advance_towards(slot.hover_amount, target, response);
        ui.card_animations
            .entry(slot.card)
            .or_default()
            .slot_hover_amount = slot.hover_amount;
        node.width = px(if slot.is_last {
            CardSize::Hand.dimensions().0
        } else {
            let reveal = shengji_hand_card_reveal(slot.hand_len);
            reveal + (HAND_CARD_HOVER_WIDTH - reveal) * slot.hover_amount
        });
    }
}

pub(crate) fn handle_shengji_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<CardDragSelection>,
    mut game_ui: ResMut<ShengjiUiState>,
    mut ui: ResMut<UiState>,
    cards: Query<(&Interaction, &RelativeCursorPosition, &ShengjiHandCardSlot)>,
) {
    let pressed = cards
        .iter()
        .find(|(interaction, _, _)| **interaction == Interaction::Pressed)
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
