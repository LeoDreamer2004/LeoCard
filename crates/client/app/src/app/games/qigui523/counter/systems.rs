use super::super::QiGui523UiState;
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_client::{ItemId, PlayerEconomy};
use leocard_protocol::GamePhaseView;

pub(in super::super) fn sync_counter(
    client: Option<Res<ClientResource>>,
    economy: Res<PlayerEconomy>,
    mut game_ui: ResMut<QiGui523UiState>,
    mut ui: ResMut<UiState>,
) {
    let state = &mut game_ui.counter;
    let model = client.as_deref().map(|client| client.0.model());
    let game = model.and_then(|model| model.qigui523_game());
    let active = game.is_some_and(|game| matches!(game.phase, GamePhaseView::Playing))
        && economy.active(ItemId::QiGui523CardCounter);
    if state.window.active != active {
        state.window.active = active;
        state.window.grab = None;
        ui.dirty = true;
    }
    if let Some((game, rules)) = game.zip(model.and_then(|model| model.qigui523_rules())) {
        state.counts.update(game, rules.deck_count);
    }
}
