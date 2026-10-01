use super::actions::dispatch_settings_actions;
use super::{
    SettingsMotion, advance_settings_modal, animate_settings_sliders,
    handle_table_appearance_sliders, poll_table_felt_picker, sync_table_appearance,
    update_settings_tab_hover, update_settings_voice_toggle_hover,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use crate::app::shell::{ModalAnimationSet, animate_cozy_modals};
use bevy::prelude::*;

pub(crate) struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsMotion>()
            .add_systems(Update, dispatch_settings_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                advance_settings_modal.in_set(ModalAnimationSet::Progress),
            )
            .add_systems(
                Update,
                (poll_table_felt_picker, handle_table_appearance_sliders)
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(Update, sync_table_appearance.in_set(ClientUpdateSet::Sync))
            .add_systems(
                Update,
                animate_settings_sliders.in_set(ClientUpdateSet::Animate),
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
