use super::{
    animate_home_arrows, dispatch_connection_actions, handle_avatar_drop, poll_avatar_picker,
    sync_avatar_images, update_home_highlights,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, dispatch_connection_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                (handle_avatar_drop, poll_avatar_picker)
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(Update, sync_avatar_images.in_set(ClientUpdateSet::Sync))
            .add_systems(Update, (update_home_highlights, animate_home_arrows));
    }
}
