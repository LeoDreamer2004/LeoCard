use super::{
    SettingsMotion, animate_cozy_modals, animate_settings_sliders, handle_table_appearance_sliders,
    poll_table_felt_picker, sync_table_appearance, update_settings_tab_hover,
    update_settings_voice_toggle_hover,
};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsMotion>()
            .add_systems(
                Update,
                (poll_table_felt_picker, handle_table_appearance_sliders)
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(Update, sync_table_appearance.in_set(ClientUpdateSet::Sync))
            .add_systems(
                Update,
                (animate_cozy_modals, animate_settings_sliders).in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    update_settings_tab_hover,
                    update_settings_voice_toggle_hover,
                )
                    .after(animate_cozy_modals)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
