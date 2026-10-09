use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_client::{LocalPlayerProfile, PlayerEconomy};
use leocard_protocol::ClientCommand;

/// 钱包是余额的唯一来源，公开档案仅用于联网展示。
pub(super) fn sync_coins(
    economy: Res<PlayerEconomy>,
    mut profile: ResMut<LocalPlayerProfile>,
    mut ui: ResMut<UiState>,
) {
    let coins = economy.coins();
    if profile.game_profiles.coins != coins {
        profile.game_profiles.coins = coins;
        ui.dirty = true;
    }
}

pub(super) fn publish_coins(
    economy: Res<PlayerEconomy>,
    mut client: Option<ResMut<ClientResource>>,
    time: Res<Time>,
    mut since_send: Local<f32>,
) {
    *since_send += time.delta_secs();
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let Some(profiles) = client.0.model().local_game_profiles() else {
        return;
    };
    if profiles.coins != economy.coins()
        && *since_send >= 0.5
        && client.0.send(ClientCommand::PublishCoins {
            coins: economy.coins(),
        })
    {
        *since_send = 0.0;
    }
}
