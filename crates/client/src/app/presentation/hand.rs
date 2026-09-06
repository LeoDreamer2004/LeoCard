//! 手牌悬停、拖选和预览输入。

use super::*;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use std::collections::HashSet;
use std::hash::Hash;

const HAND_CARD_SELECTED_LIFT: f32 = 24.0;
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

fn slot_hover_target(
    drag: &CardDragSelection,
    index: usize,
    interaction: Interaction,
    cursor: &RelativeCursorPosition,
) -> f32 {
    f32::from(if drag.active {
        cursor.cursor_over() || index == drag.current
    } else {
        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    })
}

fn advance_towards(current: f32, target: f32, response: f32) -> f32 {
    let next = current + (target - current) * response;
    if (target - next).abs() < 0.001 {
        target
    } else {
        next
    }
}

fn update_drag_selection<C>(
    mouse: &ButtonInput<MouseButton>,
    drag: &mut CardDragSelection,
    selected: &mut HashSet<C>,
    pressed: Option<(usize, C)>,
    hovered: Option<usize>,
    cards: impl Iterator<Item = (usize, C)>,
) -> bool
where
    C: Copy + Eq + Hash,
{
    if mouse.just_pressed(MouseButton::Left)
        && let Some((index, card)) = pressed
    {
        drag.active = true;
        drag.anchor = index;
        drag.current = index;
        drag.select = !selected.contains(&card);
    }
    if drag.active
        && mouse.pressed(MouseButton::Left)
        && let Some(index) = hovered
    {
        drag.current = index;
    }
    if !drag.active || !mouse.just_released(MouseButton::Left) {
        return false;
    }

    let mut changed = false;
    for (index, card) in cards {
        if !drag.contains(index) {
            continue;
        }
        changed |= if drag.select {
            selected.insert(card)
        } else {
            selected.remove(&card)
        };
    }
    drag.active = false;
    changed
}

fn drag_preview_color(drag: &CardDragSelection, index: usize) -> Color {
    if drag.contains(index) {
        Color::srgba(0.12, 0.14, 0.14, 0.52)
    } else {
        Color::NONE
    }
}

pub fn animate_hand_card_slots(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<UiState>,
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
        ui.qigui523
            .card_animations
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

pub fn animate_shengji_hand_card_slots(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<UiState>,
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
        ui.shengji
            .card_animations
            .entry(slot.card)
            .or_default()
            .slot_hover_amount = slot.hover_amount;
        node.width = px(if slot.is_last {
            CardSize::Hand.dimensions().0
        } else {
            shengji_hand_card_reveal(slot.hand_len)
                + (HAND_CARD_HOVER_WIDTH - shengji_hand_card_reveal(slot.hand_len))
                    * slot.hover_amount
        });
    }
}

pub fn handle_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<CardDragSelection>,
    mut ui: ResMut<UiState>,
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
        &mut ui.qigui523.selected,
        pressed,
        hovered,
        cards.iter().map(|(_, _, slot)| (slot.index, slot.card)),
    );
}

pub fn handle_shengji_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    mut drag: ResMut<CardDragSelection>,
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
        &mut ui.shengji.selected,
        pressed,
        hovered,
        cards.iter().map(|(_, _, slot)| (slot.index, slot.card)),
    );
}

pub fn sync_card_drag_preview(
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

pub fn sync_shengji_card_drag_preview(
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

pub fn animate_hand_cards(
    time: Res<Time>,
    drag: Res<CardDragSelection>,
    mut ui: ResMut<UiState>,
    buttons: Query<&Interaction, With<Button>>,
    mut cards: HandCardAnimations,
) {
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.72 + 0.28 * (time.elapsed_secs() * 7.0).sin();

    for (mut visual, mut transform, mut outline, mut shadow, mut image, mut border) in &mut cards {
        let Ok(interaction) = buttons.get(visual.button) else {
            continue;
        };
        let selected = ui.qigui523.selected.contains(&visual.card);
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
        // 静止且未选中的牌无需每帧重写七个 UI 组件。选中/悬停牌仍保留呼吸光效。
        if !transitioning && !hovered && !selected {
            continue;
        }
        visual.selected = selected;
        let next_hover = visual.hover_amount + (hover_target - visual.hover_amount) * response;
        visual.hover_amount = if (hover_target - next_hover).abs() < 0.001 {
            hover_target
        } else {
            next_hover
        };
        let next_selected =
            visual.selected_amount + (selected_target - visual.selected_amount) * response;
        visual.selected_amount = if (selected_target - next_selected).abs() < 0.001 {
            selected_target
        } else {
            next_selected
        };
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
            ..ui.qigui523
                .card_animations
                .get(&visual.card)
                .copied()
                .unwrap_or_default()
        };
        if ui.qigui523.card_animations.get(&visual.card).copied() != Some(next_animation) {
            ui.qigui523
                .card_animations
                .insert(visual.card, next_animation);
        }

        let hover = visual.hover_amount;
        let selected = visual.selected_amount;
        let glow = (hover * pulse + selected * 0.72).clamp(0.0, 1.0);
        let pose = hand_card_pose(
            visual.index,
            visual.hand_len,
            hover,
            selected,
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

pub fn hand_card_pose(
    index: usize,
    hand_len: usize,
    hover_amount: f32,
    selected_amount: f32,
    deal_elapsed: f32,
    dealing: bool,
) -> HandCardPose {
    let lift = hover_amount * 5.0 + selected_amount * HAND_CARD_SELECTED_LIFT;
    let deal_progress = if dealing {
        (deal_elapsed / 0.20).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let deal_progress = 1.0 - (1.0 - deal_progress).powi(3);
    let deal_offset = 1.0 - deal_progress;
    let reveal = shengji_hand_card_reveal(hand_len);
    let hand_width = if hand_len == 0 {
        0.0
    } else {
        (hand_len.saturating_sub(1) as f32 * reveal) + CardSize::Hand.dimensions().0
    };
    let final_center_x =
        index as f32 * reveal + CardSize::Hand.dimensions().0 * 0.5 - hand_width * 0.5;

    HandCardPose {
        translation: Val2::px(-final_center_x * deal_offset, -270.0 * deal_offset - lift),
        rotation: Rot2::radians(final_center_x * 0.0015 * deal_offset),
    }
}

pub fn shengji_hand_card_reveal(hand_len: usize) -> f32 {
    let card_width = CardSize::Hand.dimensions().0;
    let available_width = DESIGN_WIDTH - 96.0;
    if hand_len <= 1 {
        HAND_CARD_REVEAL
    } else {
        HAND_CARD_REVEAL.min((available_width - card_width) / (hand_len - 1) as f32)
    }
}
