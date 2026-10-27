use super::DoudizhuUiState;
use super::hand::{HandCardSelectionOverlay, HandCardSlot, HandCardVisual, hand_layout};
use crate::app::presentation::{
    HandCardHighlight, advance_towards, drag_preview_color, hand_card_pose_for_layout,
    update_drag_selection,
};
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::CardDragSelection;

pub(super) fn handle_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    client: Option<Res<ClientResource>>,
    mut drag: ResMut<CardDragSelection>,
    mut game_ui: ResMut<DoudizhuUiState>,
    mut ui: ResMut<UiState>,
    cards: Query<(&PickingInteraction, &RelativeCursorPosition, &HandCardSlot)>,
) {
    let Some(game) = client.as_deref().and_then(|c| c.0.model().doudizhu_game()) else {
        return;
    };
    if game.result.is_some()
        || game_ui.kitty_open
        || game.players.iter().any(|p| p.id == game.you && p.auto_play)
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
    ui.dirty |= update_drag_selection(
        &mouse,
        &mut drag,
        &mut game_ui.selected,
        pressed,
        hovered,
        cards.iter().map(|(_, _, slot)| (slot.index, slot.card)),
    );
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
    mut ui: ResMut<DoudizhuUiState>,
    slots: Query<&PickingInteraction, With<HandCardSlot>>,
    mut cards: Query<(
        &mut HandCardVisual,
        &mut UiTransform,
        &mut BorderColor,
        &mut Outline,
        &mut Visibility,
        &mut BoxShadow,
        &mut ImageNode,
    )>,
) {
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.72 + 0.28 * (time.elapsed_secs() * 7.0).sin();
    for (
        mut visual,
        mut transform,
        mut border,
        mut outline,
        mut visibility,
        mut shadow,
        mut image,
    ) in &mut cards
    {
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
        if !hovered
            && !selected
            && !visual.animation.dealing
            && (visual.animation.face_hover_amount - f32::from(hovered)).abs() < 0.001
            && (visual.animation.selected_amount - f32::from(selected)).abs() < 0.001
        {
            continue;
        }
        let animation = &mut visual.animation;
        animation.face_hover_amount =
            advance_towards(animation.face_hover_amount, f32::from(hovered), response);
        animation.selected_amount =
            advance_towards(animation.selected_amount, f32::from(selected), response);
        if visual.animation.dealing {
            visual.animation.deal_elapsed += time.delta_secs();
            if visual.animation.deal_elapsed >= 0.20 {
                visual.animation.dealing = false;
            }
        }
        *visibility = if visual.animation.dealing && visual.animation.deal_elapsed < 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        let pose =
            hand_card_pose_for_layout(hand_layout(visual.index, visual.hand_len), visual.animation);
        transform.translation = pose.translation;
        transform.rotation = pose.rotation;
        HandCardHighlight::new(
            visual.animation.face_hover_amount,
            visual.animation.selected_amount,
            selected,
            pulse,
        )
        .apply(&mut image, &mut border, &mut outline, &mut shadow);
        if ui.card_animations.get(&visual.card) != Some(&visual.animation) {
            ui.card_animations.insert(visual.card, visual.animation);
        }
    }
}
