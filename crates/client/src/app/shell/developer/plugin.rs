#[cfg(feature = "developer")]
use super::dispatch_developer_actions;
use super::{DeveloperHandInput, sync_developer_hand_input_text};
use crate::app::runtime::ClientUpdateSet;
#[cfg(feature = "developer")]
use crate::app::shell::UiActionSet;
use bevy::prelude::*;

pub(crate) struct DeveloperToolsPlugin;

impl Plugin for DeveloperToolsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DeveloperHandInput::default())
            .add_systems(
                Update,
                sync_developer_hand_input_text.in_set(ClientUpdateSet::Sync),
            );
        #[cfg(feature = "developer")]
        app.add_systems(Update, dispatch_developer_actions.in_set(UiActionSet));
    }
}
