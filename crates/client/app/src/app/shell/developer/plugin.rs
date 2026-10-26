use super::DeveloperHandInput;
#[cfg(feature = "developer")]
use super::submit::sync_developer_input;
use bevy::prelude::*;
#[cfg(feature = "developer")]
use leocard_ui::TextInputSet;

pub(crate) struct DeveloperToolsPlugin;

impl Plugin for DeveloperToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeveloperHandInput>();
        #[cfg(feature = "developer")]
        app.add_systems(
            PostUpdate,
            sync_developer_input.in_set(TextInputSet::Publish),
        );
    }
}
