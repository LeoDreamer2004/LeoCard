use super::{
    UpdateManager, actions::dispatch_update_actions, advance_update_modal, poll_update_events,
    sync_update_dialog,
};
use crate::app::{
    runtime::ClientUpdateSet,
    shell::{ModalAnimationSet, UiActionSet},
};
use bevy::prelude::*;

pub(crate) struct UpdatePlugin;

impl Plugin for UpdatePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_os = "android"))]
        app.add_systems(Startup, super::completion::emit_update_completion);
        app.insert_resource(UpdateManager::default())
            .add_systems(Update, dispatch_update_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                advance_update_modal.in_set(ModalAnimationSet::Progress),
            )
            .add_systems(
                Update,
                (poll_update_events, sync_update_dialog).in_set(ClientUpdateSet::Sync),
            );
    }
}
