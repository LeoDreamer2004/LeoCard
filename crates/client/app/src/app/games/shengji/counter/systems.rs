use super::super::ShengjiUiState;
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::prelude::*;
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
    if state.window.active != active {
        state.window.active = active;
        state.window.grab = None;
        ui.dirty = true;
    }
    if let Some(game) = game {
        state.counts.update(game);
    }
}
