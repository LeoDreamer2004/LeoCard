use super::super::{TexasBoardCardFlip, TexasHoldemUiState};
use super::view::TexasWinRateLabel;
use crate::app::runtime::ClientResource;
use bevy::prelude::*;

pub(in super::super) fn sync_texas_spectator(
    client: Option<Res<ClientResource>>,
    time: Res<Time>,
    mut ui: ResMut<TexasHoldemUiState>,
    flips: Query<&TexasBoardCardFlip>,
) {
    if let Some(game) = client
        .as_ref()
        .and_then(|client| client.0.model().texas_holdem_game())
    {
        ui.spectator.sync(
            game,
            time.delta_secs(),
            flips.iter().all(TexasBoardCardFlip::has_revealed),
        );
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
