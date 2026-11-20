#[cfg(feature = "desktop")]
use super::systems::setup_turn_notifications;
use super::systems::{TurnNotifications, update_turn_notifications};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(in crate::app::runtime) struct TurnNotificationPlugin;

impl Plugin for TurnNotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TurnNotifications>().add_systems(
            Update,
            update_turn_notifications.in_set(ClientUpdateSet::Animate),
        );
        #[cfg(feature = "desktop")]
        app.add_systems(Startup, setup_turn_notifications);
    }
}
