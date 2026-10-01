use super::actions::dispatch_update_actions;
use super::advance_update_modal;
use super::{UpdateManager, poll_update_events, sync_update_dialog};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{ModalAnimationSet, UiActionSet};
use bevy::prelude::*;

pub(crate) struct UpdatePlugin;

impl Plugin for UpdatePlugin {
    fn build(&self, app: &mut App) {
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
