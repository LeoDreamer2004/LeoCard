use super::BlackjackPlayback;
use crate::app::presentation::PendingDealSound;
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiState;
use leocard_protocol::GameSnapshot;

use bevy::prelude::*;

pub(crate) fn sync_blackjack_playback(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut playback: ResMut<BlackjackPlayback>,
    mut shell: ResMut<UiState>,
) {
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::blackjack)
    }) else {
        *playback = BlackjackPlayback::default();
        return;
    };
    let was_busy = playback.busy();
    let was_settling = playback.settlement_started();
    playback.elapsed += time.delta_secs();
    for (index, remaining) in playback.observe(game).into_iter().enumerate() {
        commands.spawn(PendingDealSound {
            remaining,
            variant: index % assets.audio.deal_sounds.len().max(1),
        });
    }
    if was_busy != playback.busy() || was_settling != playback.settlement_started() {
        shell.dirty = true;
    }
}
