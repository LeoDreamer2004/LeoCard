use crate::app::achievements::LocalAchievementTrigger;
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_achievements::{AchievementTrigger, PersonalEvent};
use leocard_client::{LocalPlayerProfile, PlayerEconomy};
use leocard_protocol::ClientCommand;

/// 钱包提供余额和已选头像框，公开档案用于展示。
pub(super) fn sync_economy(
    economy: Res<PlayerEconomy>,
    mut profile: ResMut<LocalPlayerProfile>,
    mut ui: ResMut<UiState>,
    mut observed: Local<Option<u32>>,
    mut achievements: MessageWriter<LocalAchievementTrigger>,
) {
    let coins = economy.coins();
    if *observed != Some(coins) {
        *observed = Some(coins);
        achievements.write(LocalAchievementTrigger(AchievementTrigger::Personal(
            PersonalEvent::CoinBalance { coins },
        )));
    }
    let avatar_frame = economy.equipped_frame();
    if profile.game_profiles.coins != coins || profile.game_profiles.avatar_frame != avatar_frame {
        profile.game_profiles.coins = coins;
        profile.game_profiles.avatar_frame = avatar_frame;
        ui.dirty = true;
    }
}

pub(super) fn publish_economy(
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
    let coins = economy.coins();
    let avatar_frame = economy.equipped_frame();
    if (profiles.coins != coins || profiles.avatar_frame != avatar_frame)
        && *since_send >= 0.5
        && client.0.send(ClientCommand::PublishEconomy {
            coins,
            avatar_frame,
        })
    {
        *since_send = 0.0;
    }
}
