use super::{HandCardSlot, HandCardVisual, QiGui523UiState};
use crate::app::{
    ACCENT, BORDER, CardAnimationState, CardDragSelection, CardSize, HAND_CARD_REVEAL,
    HandCardSelectionOverlay, advance_towards, drag_preview_color, hand_card_pose,
    slot_hover_target, update_drag_selection,
};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

const HAND_CARD_HOVER_WIDTH: f32 = 32.0;

type HandCardAnimations<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut HandCardVisual,
        &'static mut UiTransform,
        &'static mut Outline,
        &'static mut BoxShadow,
        &'static mut ImageNode,
        &'static mut BorderColor,
    ),
>;

pub(super) fn animate_hand_card_slots(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<QiGui523UiState>,
    mut cards: Query<
        (
            &Interaction,
            &RelativeCursorPosition,
            &mut HandCardSlot,
            &mut Node,
        ),
        With<Button>,
    >,
) {
    let response = 1.0 - (-18.0 * time.delta_secs()).exp();
    for (interaction, cursor, mut slot, mut node) in &mut cards {
        let target = slot_hover_target(&drag, slot.index, *interaction, cursor);
        if (target - slot.hover_amount).abs() < 0.001 {
            continue;
        }
        slot.hover_amount = advance_towards(slot.hover_amount, target, response);
        ui.card_animations
            .entry(slot.card)
            .or_default()
            .slot_hover_amount = slot.hover_amount;
        node.width = px(if slot.is_last {
            CardSize::Hand.dimensions().0
        } else {
            HAND_CARD_REVEAL + (HAND_CARD_HOVER_WIDTH - HAND_CARD_REVEAL) * slot.hover_amount
        });
    }
}

pub(super) fn handle_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<CardDragSelection>,
    mut ui: ResMut<QiGui523UiState>,
    cards: Query<(&Interaction, &RelativeCursorPosition, &HandCardSlot)>,
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
    update_drag_selection(
        &mouse,
        &mut drag,
        &mut ui.selected,
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
        let expected = drag_preview_color(&drag, overlay.index);
        if color.0 != expected {
            color.0 = expected;
        }
    }
}

pub(super) fn animate_hand_cards(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<QiGui523UiState>,
    buttons: Query<&Interaction, With<Button>>,
    mut cards: HandCardAnimations,
) {
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.72 + 0.28 * (time.elapsed_secs() * 7.0).sin();

    for (mut visual, mut transform, mut outline, mut shadow, mut image, mut border) in &mut cards {
        let Ok(interaction) = buttons.get(visual.button) else {
            continue;
        };
        let selected = ui.selected.contains(&visual.card);
        let hovered = if drag.active {
            visual.index == drag.current
        } else {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        };
        let hover_target = f32::from(hovered);
        let selected_target = f32::from(selected);
        let transitioning = (hover_target - visual.hover_amount).abs() >= 0.001
            || (selected_target - visual.selected_amount).abs() >= 0.001
            || visual.selected != selected
            || visual.dealing;
        if !transitioning && !hovered && !selected {
            continue;
        }
        visual.selected = selected;
        visual.hover_amount = advance_towards(visual.hover_amount, hover_target, response);
        visual.selected_amount = advance_towards(visual.selected_amount, selected_target, response);
        if visual.dealing {
            visual.deal_elapsed += time.delta_secs();
            if visual.deal_elapsed >= 0.20 {
                visual.dealing = false;
            }
        }
        let next_animation = CardAnimationState {
            face_hover_amount: visual.hover_amount,
            selected_amount: visual.selected_amount,
            deal_elapsed: visual.deal_elapsed,
            dealing: visual.dealing,
            ..ui.card_animations
                .get(&visual.card)
                .copied()
                .unwrap_or_default()
        };
        if ui.card_animations.get(&visual.card).copied() != Some(next_animation) {
            ui.card_animations.insert(visual.card, next_animation);
        }

        let glow = (visual.hover_amount * pulse + visual.selected_amount * 0.72).clamp(0.0, 1.0);
        let pose = hand_card_pose(
            visual.index,
            visual.hand_len,
            visual.hover_amount,
            visual.selected_amount,
            visual.deal_elapsed,
            visual.dealing,
        );
        transform.scale = Vec2::ONE;
        transform.translation = pose.translation;
        transform.rotation = pose.rotation;
        outline.width = px(0.75 + glow * 1.5);
        outline.offset = px(0);
        outline.color = ACCENT.with_alpha(glow * 0.92);
        image.color = Color::srgb(1.0, 1.0 - glow * 0.035, 1.0 - glow * 0.16);
        border.set_all(if visual.selected { ACCENT } else { BORDER });
        if let Some(style) = shadow.0.first_mut() {
            style.color = ACCENT.with_alpha(glow * 0.58);
            style.spread_radius = px(glow * 2.0);
            style.blur_radius = px(2.0 + glow * 10.0);
        }
    }
}
