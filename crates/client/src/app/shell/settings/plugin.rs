use super::{handle_table_appearance_sliders, poll_table_felt_picker, sync_table_appearance};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (poll_table_felt_picker, handle_table_appearance_sliders)
                .chain()
                .in_set(ClientUpdateSet::Input),
        )
        .add_systems(Update, sync_table_appearance.in_set(ClientUpdateSet::Sync));
    }
}
