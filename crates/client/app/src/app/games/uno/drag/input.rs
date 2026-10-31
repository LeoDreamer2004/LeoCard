use super::state::UnoDragGesture;
use super::{UnoDragState, UnoDraggableCard};
use crate::app::games::uno::{UnoDiscardPileAnchor, UnoHandCardVisual, UnoUiAction, UnoUiState};
use crate::app::runtime::ClientResource;
use crate::app::shell::{ConfirmationDialog, PageMotion, PressedUiAction, UiAction, UiState};
use bevy::camera::NormalizedRenderTarget;
use bevy::ecs::system::SystemParam;
#[cfg(target_os = "android")]
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::picking::events::PointerPress;
use bevy::picking::pointer::{PointerAction, PointerButton, PointerInput};
use bevy::prelude::*;
use leocard_client::NetworkState;
use leocard_protocol::GameSnapshot;

#[derive(SystemParam)]
pub(in super::super) struct UnoDragInput<'w, 's> {
    client: Option<Res<'w, ClientResource>>,
    game_ui: Res<'w, UnoUiState>,
    ui: ResMut<'w, UiState>,
    confirmation: Res<'w, ConfirmationDialog>,
    motion: Res<'w, PageMotion>,
    scale: Res<'w, UiScale>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    windows: Query<'w, 's, &'static Window>,
    buttons: Query<'w, 's, &'static UnoDraggableCard>,
    faces: Query<'w, 's, (&'static UnoHandCardVisual, &'static UiGlobalTransform)>,
    discards: Query<'w, 's, &'static UiGlobalTransform, With<UnoDiscardPileAnchor>>,
    #[cfg(target_os = "android")]
    touches: MessageReader<'w, 's, TouchInput>,
}

impl UnoDragInput<'_, '_> {
    fn blocked(&self) -> bool {
        self.game_ui.color_choice.is_some()
            || self.ui.settings.open
            || self.ui.shop.open
            || self.ui.profile.open
            || self.ui.leaving_room
            || self.confirmation.is_open()
            || self.motion.active()
            || self.keys.just_pressed(KeyCode::Escape)
    }
}

pub(in super::super) fn update_uno_drag(
    mut presses: MessageReader<PointerPress>,
    mut inputs: MessageReader<PointerInput>,
    mut drag: ResMut<UnoDragState>,
    mut actions: MessageWriter<PressedUiAction>,
    mut context: UnoDragInput,
) {
    // Android adapts a canceled finger to a mouse release; it must never play a card.
    #[cfg(target_os = "android")]
    if context
        .touches
        .read()
        .any(|touch| touch.phase == TouchPhase::Canceled)
    {
        drag.gesture = None;
        presses.clear();
        inputs.clear();
        return;
    }
    let game = context.client.as_deref().and_then(|client| {
        matches!(client.0.state(), NetworkState::Connected(_))
            .then(|| client.0.model().game_snapshot().and_then(GameSnapshot::uno))
            .flatten()
    });
    let Some(game) = game else {
        drag.gesture = None;
        presses.clear();
        inputs.clear();
        return;
    };
    if context.blocked() {
        drag.gesture = None;
        presses.clear();
        inputs.clear();
        return;
    }
    if drag.gesture.as_ref().is_some_and(|gesture| {
        !gesture.valid(game)
            || gesture.scale != context.scale.0
            || !context
                .windows
                .get(gesture.window)
                .is_ok_and(|window| window.focused)
    }) {
        drag.gesture = None;
    }

    let mut awaiting_press = false;
    for press in presses.read() {
        if press.button != PointerButton::Primary || drag.gesture.is_some() {
            continue;
        }
        let Ok(card) = context.buttons.get(press.entity) else {
            continue;
        };
        let NormalizedRenderTarget::Window(window_ref) = press.pointer.target else {
            continue;
        };
        let Ok(window) = context.windows.get(window_ref.entity()) else {
            continue;
        };
        let Some((_, face)) = context
            .faces
            .iter()
            .find(|(visual, _)| visual.card == card.0)
        else {
            continue;
        };
        let Ok(discard) = context.discards.single() else {
            continue;
        };
        if !UnoDragGesture::can_drag(game, card.0) {
            continue;
        }
        let scale = context.scale.0;
        let origin = face.to_scale_angle_translation().2 / (window.scale_factor() * scale);
        let cursor = press.pointer.position / scale;
        awaiting_press = true;
        drag.gesture = Some(UnoDragGesture {
            card: card.0,
            match_id: game.match_id,
            discard: game.discard_top,
            current_player: game.current_player,
            pointer: press.pointer.id,
            window: window_ref.entity(),
            scale,
            press: cursor,
            cursor,
            origin,
            target: discard.to_scale_angle_translation().2 / (window.scale_factor() * scale),
            dragging: false,
        });
    }

    for input in inputs.read() {
        let Some(gesture) = drag.gesture.as_mut() else {
            continue;
        };
        if input.pointer_id != gesture.pointer {
            continue;
        }
        // Earlier moves in the press frame belong to hover, not this gesture.
        if awaiting_press {
            awaiting_press = !input.button_just_pressed(PointerButton::Primary);
            continue;
        }
        gesture.cursor = input.location.position / gesture.scale;
        gesture.dragging |= gesture.cursor.distance(gesture.press) > 8.0;
        match input.action {
            PointerAction::Release(PointerButton::Primary) => {
                if gesture.dragging && gesture.ready() {
                    actions.write(PressedUiAction(UiAction::Uno(UnoUiAction::DragCard(
                        gesture.card,
                    ))));
                    context.ui.dirty = true;
                }
                drag.gesture = None;
            }
            PointerAction::Cancel | PointerAction::Press(PointerButton::Secondary) => {
                drag.gesture = None;
            }
            _ => {}
        }
    }
}
