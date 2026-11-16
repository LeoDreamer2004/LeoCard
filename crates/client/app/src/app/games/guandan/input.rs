use super::hand::{HandCardSelectionOverlay, HandCardSlot, HandCardVisual};
use super::hit_testing::{CardHitRegion, crossed_cards};
use super::state::GuandanUiState;
use crate::app::games::HandSelection;
use crate::app::presentation::{HandCardHighlight, advance_towards, drag_preview_color};
use crate::app::runtime::ClientResource;
use crate::app::shell::{ConfirmationDialog, PageMotion, UiState};
use bevy::ecs::system::SystemParam;
use bevy::picking::events::{PointerCancel, PointerClick};
use bevy::picking::hover::PickingInteraction;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;
use leocard_guandan::GuandanCard;
use leocard_protocol::GameSnapshot;
use leocard_ui::CardDragSelection;

#[derive(SystemParam)]
pub(super) struct HandPointerInput<'w, 's> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    cards: Query<
        'w,
        's,
        (
            Entity,
            &'static PickingInteraction,
            &'static RelativeCursorPosition,
            &'static HandCardSlot,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
    clicks: MessageReader<'w, 's, PointerClick>,
    cancels: MessageReader<'w, 's, PointerCancel>,
    confirmation: Res<'w, ConfirmationDialog>,
    motion: Res<'w, PageMotion>,
}

#[derive(Default)]
pub(super) struct HandDragGesture {
    origin: Option<Vec2>,
    previous: Option<Vec2>,
    hand: Vec<GuandanCard>,
    moved: bool,
}

pub(super) fn handle_card_drag_selection(
    mut input: HandPointerInput,
    client: Option<Res<ClientResource>>,
    mut drag: ResMut<CardDragSelection>,
    mut game_ui: ResMut<GuandanUiState>,
    mut ui: ResMut<UiState>,
    mut gesture: Local<HandDragGesture>,
) {
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::guandan)
    }) else {
        input.clicks.clear();
        input.cancels.clear();
        return;
    };
    let canceled = input.cancels.read().count() > 0;
    let window = input.windows.single().ok();
    if client
        .as_deref()
        .is_none_or(|client| client.0.model().you() != Some(game.you))
        || game.result.is_some()
        || game_ui.is_dealing()
        || game
            .players
            .iter()
            .any(|player| player.player.id == game.you && player.auto_play)
        || window.is_none_or(|window| !window.focused)
        || canceled
        || input.confirmation.is_open()
        || input.motion.active()
        || ui.settings.open
        || ui.shop.open
        || ui.profile.open
        || ui.leaving_room
    {
        drag.active = false;
        input.clicks.clear();
        return;
    }
    let cursor = window.and_then(|window| {
        window
            .cursor_position()
            .map(|point| point * window.scale_factor())
    });
    let mut current_hand = input
        .cards
        .iter()
        .map(|(_, _, _, slot, _, _)| (slot.index, slot.card))
        .collect::<Vec<_>>();
    current_hand.sort_by_key(|(index, _)| *index);
    let current_hand = current_hand
        .into_iter()
        .map(|(_, card)| card)
        .collect::<Vec<_>>();
    if drag.active
        && (gesture.hand != current_hand
            || gesture.hand.len() != game.your_hand.len()
            || gesture
                .hand
                .iter()
                .any(|card| !game.your_hand.contains(card)))
    {
        drag.active = false;
    }
    if input.mouse.just_pressed(MouseButton::Left)
        && let Some((_, _, _, slot, _, _)) =
            input
                .cards
                .iter()
                .find(|(_, interaction, cursor, _, _, _)| {
                    **interaction == PickingInteraction::Pressed && cursor.cursor_over()
                })
    {
        drag.begin_path(slot.index, !game_ui.selected.contains(&slot.card));
        gesture.origin = cursor;
        gesture.previous = cursor;
        gesture.hand = current_hand;
        gesture.moved = false;
    }
    if !drag.active {
        input.clicks.clear();
        return;
    }
    if let Some(cursor) = cursor {
        let start = gesture.previous.unwrap_or(cursor);
        gesture.moved |= gesture
            .origin
            .is_some_and(|origin| origin.distance(cursor) > 6.0);
        let regions = input
            .cards
            .iter()
            .map(|(_, _, _, slot, node, transform)| {
                let half = node.size() * 0.5;
                CardHitRegion {
                    index: slot.index,
                    rect: Rect::from_corners(
                        transform.transform_point2(-half),
                        transform.transform_point2(half),
                    ),
                }
            })
            .collect::<Vec<_>>();
        for index in crossed_cards(start, cursor, &regions) {
            drag.visit(index);
        }
        if let Some((_, _, _, slot, _, _)) =
            input
                .cards
                .iter()
                .find(|(_, interaction, relative, _, _, _)| {
                    matches!(
                        **interaction,
                        PickingInteraction::Hovered | PickingInteraction::Pressed
                    ) && relative.cursor_over()
                })
        {
            drag.visit(slot.index);
            gesture.moved |= slot.index != drag.anchor;
        }
        gesture.previous = Some(cursor);
    }
    if !input.mouse.just_released(MouseButton::Left) {
        input.clicks.clear();
        return;
    }
    let mut changed = drag.finish(
        &mut game_ui.selected,
        input
            .cards
            .iter()
            .map(|(_, _, _, slot, _, _)| (slot.index, slot.card)),
    );
    for click in input.clicks.read() {
        if gesture.moved || click.button != PointerButton::Primary || !click.count.is_multiple_of(2)
        {
            continue;
        }
        let Ok((_, _, _, clicked, _, _)) = input.cards.get(click.entity) else {
            continue;
        };
        let column = input
            .cards
            .iter()
            .filter(|(_, _, _, slot, _, _)| slot.column == clicked.column)
            .map(|(_, _, _, slot, _, _)| slot.card)
            .collect::<Vec<_>>();
        changed |= game_ui.toggle_selection(column);
    }
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
    mut shell_ui: ResMut<UiState>,
    slots: Query<&PickingInteraction, With<HandCardSlot>>,
    mut cards: Query<(
        &mut HandCardVisual,
        &mut UiTransform,
        &mut BorderColor,
        &mut Outline,
        &mut BoxShadow,
        &mut ImageNode,
        &mut Visibility,
    )>,
) {
    let was_dealing = ui.is_dealing();
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.72 + 0.28 * (time.elapsed_secs() * 7.0).sin();
    for (
        mut visual,
        mut transform,
        mut border,
        mut outline,
        mut shadow,
        mut image,
        mut visibility,
    ) in &mut cards
    {
        let hovered = if drag.active {
            visual.placement.index == drag.current
        } else {
            matches!(
                slots.get(visual.button),
                Ok(PickingInteraction::Hovered | PickingInteraction::Pressed)
            )
        };
        let selected = ui.selected.contains(&visual.card);
        let card = visual.card;
        let placement = visual.placement;
        let animation = &mut visual.animation;
        animation.face_hover_amount =
            advance_towards(animation.face_hover_amount, f32::from(hovered), response);
        animation.selected_amount =
            advance_towards(animation.selected_amount, f32::from(selected), response);
        if animation.dealing {
            animation.deal_elapsed += time.delta_secs();
            animation.dealing = animation.deal_elapsed < 0.20;
            *visibility = if animation.deal_elapsed < 0.0 {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
        let pose = placement.pose(*animation);
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
    if was_dealing && !ui.is_dealing() {
        shell_ui.dirty = true;
    }
}
