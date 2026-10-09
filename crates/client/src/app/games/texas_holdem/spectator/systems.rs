use super::super::{TexasBoardCardFlip, TexasHoldemUiState};
use super::view::TexasWinRateLabel;
use crate::app::runtime::ClientResource;
use crate::app::shell::{UiState, game_command};
use bevy::prelude::*;
use leocard_client::{ItemId, PlayerEconomy};
use leocard_protocol::TexasHoldemCommand;

pub(in super::super) fn sync_texas_spectator(
    mut client: Option<ResMut<ClientResource>>,
    time: Res<Time>,
    mut ui: ResMut<TexasHoldemUiState>,
    flips: Query<&TexasBoardCardFlip>,
    economy: Res<PlayerEconomy>,
    mut shell: ResMut<UiState>,
) {
    if let Some(client) = client.as_deref_mut()
        && let Some(game) = client.0.model().texas_holdem_game()
    {
        let was_active = ui.spectator.item_active;
        ui.spectator.sync(
            game,
            time.delta_secs(),
            flips.iter().all(TexasBoardCardFlip::has_revealed),
        );
        let state = &mut ui.spectator;
        state.item_active = economy.active(ItemId::ObservationLens);
        if was_active != state.item_active {
            shell.dirty = true;
        }
        let enabled = state.item_active && state.preferences.show_win_rates;
        if enabled == game.spectator_win_rates_enabled {
            state.pending_request = None;
        } else if state.pending_request != Some(enabled) {
            state.pending_request = Some(enabled);
            client
                .0
                .send(game_command(TexasHoldemCommand::SetSpectatorWinRates {
                    enabled,
                }));
        }
    } else {
        ui.spectator.clear();
    }
}

pub(in super::super) fn animate_texas_win_rates(
    ui: Res<TexasHoldemUiState>,
    mut labels: Query<(&TexasWinRateLabel, &mut Text)>,
) {
    for (label, mut text) in &mut labels {
        let value = ui
            .spectator
            .displayed(label.0)
            .map_or_else(|| "—%".to_owned(), |rate| format!("{rate:.0}%"));
        if text.0 != value {
            text.0 = value;
        }
    }
}
