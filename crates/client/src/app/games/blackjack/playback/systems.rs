use super::BlackjackPlayback;
use crate::app::presentation::PendingDealSound;
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiState;
use bevy::prelude::*;

pub(crate) fn sync_blackjack_playback(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut playback: ResMut<BlackjackPlayback>,
    mut shell: ResMut<UiState>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().blackjack_game())
    else {
        *playback = BlackjackPlayback::default();
        return;
    };
    let was_busy = playback.busy();
    playback.elapsed += time.delta_secs();
    for (index, remaining) in playback.observe(game).into_iter().enumerate() {
        commands.spawn(PendingDealSound {
            remaining,
            variant: index % assets.audio.deal_sounds.len().max(1),
        });
    }
    if was_busy != playback.busy() {
        shell.dirty = true;
    }
}
