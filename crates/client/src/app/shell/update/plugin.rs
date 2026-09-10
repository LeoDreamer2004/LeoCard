use super::{UpdateManager, poll_update_events, sync_update_dialog};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct UpdatePlugin;

impl Plugin for UpdatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UpdateManager::default()).add_systems(
            Update,
            (poll_update_events, sync_update_dialog).in_set(ClientUpdateSet::Sync),
        );
    }
}
