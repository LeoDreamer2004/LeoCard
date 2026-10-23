use super::{super::ShengjiUiState, state::MissingSuitMarker};
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_client::{ItemId, PlayerEconomy};
use leocard_protocol::ShengjiPhaseView;
use std::collections::hash_map::Entry;

pub(in super::super) fn sync_missing_suits(
    client: Option<Res<ClientResource>>,
    economy: Res<PlayerEconomy>,
    time: Res<Time>,
    mut game_ui: ResMut<ShengjiUiState>,
    mut ui: ResMut<UiState>,
) {
    let state = &mut game_ui.missing_suits;
    let game = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game());
    let active = game.is_some_and(|game| {
        matches!(game.phase, ShengjiPhaseView::Playing)
            && game.players.iter().any(|player| player.hand_len > 0)
            && economy.active(ItemId::ShengjiMissingSuitCard)
    });
    if state.active != active {
        state.active = active;
        ui.dirty = true;
    }
    let hand = game.map(|game| (game.match_id, game.hand_number));
    if state.hand != hand {
        state.hand = hand;
        state.ages.clear();
    }
    for age in state.ages.values_mut() {
        *age = (*age + time.delta_secs()).min(0.24);
    }
    if let Some(game) = game.filter(|_| active) {
        for player in game.players.iter().filter(|player| player.id != game.you) {
            for &door in &player.missing_suits {
                if let Entry::Vacant(entry) = state.ages.entry((player.id, door)) {
                    entry.insert(0.0);
                    ui.dirty = true;
                }
            }
        }
    }
}

pub(in super::super) fn animate_missing_suits(
    ui: Res<ShengjiUiState>,
    mut markers: Query<(&MissingSuitMarker, &mut UiTransform)>,
) {
    for (marker, mut transform) in &mut markers {
        let age = ui
            .missing_suits
            .ages
            .get(&(marker.player, marker.door))
            .copied()
            .unwrap_or(0.24);
        let remaining = (1.0 - age / 0.24).clamp(0.0, 1.0);
        *transform = UiTransform::from_translation(Val2::px(0.0, 14.0 * remaining.powi(3)));
    }
}
