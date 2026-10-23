use super::{
    AchievementPublication, AchievementSession, messages::*, notifications::*, processing::*,
    service::*,
};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct AchievementPlugin;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub(crate) struct AchievementUpdateSet;

impl Plugin for AchievementPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<AchievementUnlocked>()
            .add_message::<LocalAchievementTrigger>()
            .init_resource::<AchievementSession>()
            .init_resource::<AchievementPublication>()
            .init_resource::<ActivityObserver>()
            .init_resource::<AchievementNotifications>()
            .add_systems(Startup, (setup_book, setup_notifications))
            .add_systems(
                Update,
                (
                    reconcile_connection,
                    process_triggers,
                    commit_progress,
                    publish_progress,
                    queue_achievement_notifications,
                )
                    .chain()
                    .in_set(AchievementUpdateSet)
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                animate_achievement_notifications.in_set(ClientUpdateSet::Animate),
            );
    }
}
