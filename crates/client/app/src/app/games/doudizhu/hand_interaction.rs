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
use leocard_doudizhu::{DoudizhuCard, DoudizhuRuleSet, classify};
use leocard_protocol::GameSnapshot;
use leocard_ui::CardDragSelection;
use std::collections::HashSet;

pub(super) fn handle_card_drag_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    client: Option<Res<ClientResource>>,
    mut drag: ResMut<CardDragSelection>,
    mut game_ui: ResMut<DoudizhuUiState>,
    mut ui: ResMut<UiState>,
    cards: Query<(&PickingInteraction, &RelativeCursorPosition, &HandCardSlot)>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|c| c.0.model().game_snapshot().and_then(GameSnapshot::doudizhu))
    else {
        return;
    };
    if game.result.is_some()
        || game_ui.kitty_open
        || game
            .players
            .iter()
            .any(|p| p.player.id == game.you && p.auto_play)
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
    let multi_card_select = mouse.just_released(MouseButton::Left)
        && drag.active
        && drag.select
        && drag.anchor != drag.current;
    let changed = update_drag_selection(
        &mouse,
        &mut drag,
        &mut game_ui.selected,
        pressed,
        hovered,
        cards.iter().map(|(_, _, slot)| (slot.index, slot.card)),
    );
    if multi_card_select && changed {
        refine_drag_selection(&mut game_ui.selected, game.rules);
    }
    ui.dirty |= changed;
}

pub(super) fn refine_drag_selection(selected: &mut HashSet<DoudizhuCard>, rules: DoudizhuRuleSet) {
    let cards: Vec<_> = selected.iter().copied().collect();
    if classify(&cards, rules).is_some() {
        return;
    }
    retain_largest_straight(selected);
}

pub(super) fn retain_largest_straight(selected: &mut HashSet<DoudizhuCard>) {
    let mut by_rank: [Option<DoudizhuCard>; 12] = [None; 12];
    for &card in selected.iter() {
        let rank = usize::from(card.rank());
        if rank < by_rank.len() {
            by_rank[rank] = Some(by_rank[rank].map_or(card, |current| current.min(card)));
        }
    }

    let mut best = None;
    let mut start = 0;
    while start < by_rank.len() {
        if by_rank[start].is_none() {
            start += 1;
            continue;
        }
        let mut end = start + 1;
        while end < by_rank.len() && by_rank[end].is_some() {
            end += 1;
        }
        if end - start >= 5
            && best.is_none_or(|(best_start, best_end)| {
                (end - start, end) > (best_end - best_start, best_end)
            })
        {
            best = Some((start, end));
        }
        start = end;
    }

    let Some((start, end)) = best else {
        return;
    };
    selected.retain(|card| {
        let rank = usize::from(card.rank());
        (start..end).contains(&rank) && by_rank[rank] == Some(*card)
    });
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
