use super::{
    super::ShengjiUiState,
    state::{CounterDragHandle, CounterWindow},
};
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::{prelude::*, ui::RelativeCursorPosition, window::PrimaryWindow};
use leocard_client::{ItemId, PlayerEconomy};
use leocard_protocol::ShengjiPhaseView;

pub(in super::super) fn sync_counter(
    client: Option<Res<ClientResource>>,
    economy: Res<PlayerEconomy>,
    mut game_ui: ResMut<ShengjiUiState>,
    mut ui: ResMut<UiState>,
) {
    let state = &mut game_ui.counter;
    let game = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game());
    let active = game.is_some_and(|game| {
        matches!(game.phase, ShengjiPhaseView::Playing)
            && game.players.iter().any(|player| player.hand_len > 0)
            && economy.active(ItemId::ShengjiCardCounter)
    });
    if state.active != active {
        state.active = active;
        state.grab = None;
        ui.dirty = true;
    }
    if let Some(game) = game {
        state.counts.update(game);
    }
}

pub(in super::super) fn drag_counter_window(
    mouse: Res<ButtonInput<MouseButton>>,
    scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    handles: Query<&RelativeCursorPosition, With<CounterDragHandle>>,
    mut panels: Query<(&mut Node, &ComputedNode), With<CounterWindow>>,
    mut ui: ResMut<ShengjiUiState>,
) {
    let state = &mut ui.counter;
    if !mouse.pressed(MouseButton::Left) || panels.is_empty() {
        state.grab = None;
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let cursor = cursor / scale.0;
    if mouse.just_pressed(MouseButton::Left) && handles.iter().any(|handle| handle.cursor_over()) {
        state.grab = Some(cursor - state.position);
    }
    let Some(grab) = state.grab else {
        return;
    };
    for (mut node, computed) in &mut panels {
        let viewport = Vec2::new(window.width(), window.height()) / scale.0;
        let size = computed.size() * computed.inverse_scale_factor();
        state.position = (cursor - grab).clamp(Vec2::ZERO, (viewport - size).max(Vec2::ZERO));
        node.left = px(state.position.x);
        node.top = px(state.position.y);
    }
}
