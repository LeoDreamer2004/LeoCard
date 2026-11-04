use super::{notifications::*, rewards::*, sync::*};
use crate::app::achievements::AchievementUpdateSet;
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingRewards>()
            .add_systems(PostStartup, setup_rewards)
            .add_systems(
                Update,
                (process_rewards, sync_coins, publish_coins)
                    .chain()
                    .after(AchievementUpdateSet)
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                queue_coin_notifications.in_set(ClientUpdateSet::Sync),
            );
    }
}
