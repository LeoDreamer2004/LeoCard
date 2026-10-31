use super::DoudizhuUiState;
use crate::app::presentation::PendingDealSound;
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_protocol::GameSnapshot;

pub(super) fn queue_deal_animations(
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut state: ResMut<DoudizhuUiState>,
    mut ui: ResMut<UiState>,
    mut commands: Commands,
) {
    let Some(game) = client.as_deref().and_then(|client| {
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::doudizhu)
    }) else {
        return;
    };
    state.reconcile(game);
    let deals = state.observe_dealt_hand(game.match_id, &game.your_hand);
    if !deals.is_empty() {
        ui.dirty = true;
    }
    for (_, delay) in deals {
        if !assets.audio.deal_sounds.is_empty() {
            commands.spawn(PendingDealSound {
                remaining: delay,
                variant: fastrand::usize(..assets.audio.deal_sounds.len()),
            });
        }
    }
}
