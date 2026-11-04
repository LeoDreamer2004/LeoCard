use super::hand::{HandCardSelectionOverlay, HandCardSlot, HandCardVisual, hand_layout};
use super::state::GuandanUiState;
use crate::app::presentation::{
    HandCardHighlight, advance_towards, drag_preview_color, hand_card_pose_for_layout,
    update_drag_selection,
};
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_protocol::GameSnapshot;
use leocard_ui::CardDragSelection;

pub(super) fn handle_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    client: Option<Res<ClientResource>>,
    mut drag: ResMut<CardDragSelection>,
    mut game_ui: ResMut<GuandanUiState>,
    mut ui: ResMut<UiState>,
    cards: Query<(&PickingInteraction, &RelativeCursorPosition, &HandCardSlot)>,
) {
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::guandan)
    }) else {
        return;
    };
    if game.result.is_some()
        || game
            .players
            .iter()
            .any(|player| player.player.id == game.you && player.auto_play)
    {
        drag.active = false;
        return;
    }
    let pressed = cards
        .iter()
        .find(|(interaction, _, _)| **interaction == PickingInteraction::Pressed)
        .map(|(_, _, slot)| (slot.index, slot.card));
    let hovered = cards
        .iter()
        .filter(|(_, cursor, _)| cursor.cursor_over())
        .map(|(_, _, slot)| slot.index)
        .max();
    let changed = update_drag_selection(
        &mouse,
        &mut drag,
        &mut game_ui.selected,
        pressed,
        hovered,
        cards.iter().map(|(_, _, slot)| (slot.index, slot.card)),
    );
    ui.dirty |= changed;
}

pub(super) fn sync_card_drag_preview(
    drag: Res<CardDragSelection>,
    mut overlays: Query<(&HandCardSelectionOverlay, &mut BackgroundColor)>,
) {
    if !drag.is_changed() {
        return;
    }
    for (overlay, mut color) in &mut overlays {
        color.0 = drag_preview_color(&drag, overlay.index);
    }
}

pub(super) fn animate_hand_cards(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<GuandanUiState>,
    slots: Query<&PickingInteraction, With<HandCardSlot>>,
    mut cards: Query<(
        &mut HandCardVisual,
        &mut UiTransform,
        &mut BorderColor,
        &mut Outline,
        &mut BoxShadow,
        &mut ImageNode,
    )>,
) {
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.72 + 0.28 * (time.elapsed_secs() * 7.0).sin();
    for (mut visual, mut transform, mut border, mut outline, mut shadow, mut image) in &mut cards {
        let Ok(interaction) = slots.get(visual.button) else {
            continue;
        };
        let hovered = if drag.active {
            visual.index == drag.current
        } else {
            matches!(
                *interaction,
                PickingInteraction::Hovered | PickingInteraction::Pressed
            )
        };
        let selected = ui.selected.contains(&visual.card);
        let card = visual.card;
        let index = visual.index;
        let hand_len = visual.hand_len;
        let animation = &mut visual.animation;
        animation.face_hover_amount =
            advance_towards(animation.face_hover_amount, f32::from(hovered), response);
        animation.selected_amount =
            advance_towards(animation.selected_amount, f32::from(selected), response);
        let pose = hand_card_pose_for_layout(hand_layout(index, hand_len), *animation);
        transform.translation = pose.translation;
        transform.rotation = pose.rotation;
        HandCardHighlight::new(
            animation.face_hover_amount,
            animation.selected_amount,
            selected,
            pulse,
        )
        .apply(&mut image, &mut border, &mut outline, &mut shadow);
        if ui.card_animations.get(&card) != Some(animation) {
            ui.card_animations.insert(card, *animation);
        }
    }
}
