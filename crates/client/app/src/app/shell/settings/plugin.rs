use super::actions::dispatch_settings_actions;
#[cfg(not(target_arch = "wasm32"))]
use super::ai::sync_ai_settings_inputs;
use super::{
    SettingsMotion, advance_settings_modal, animate_settings_sliders,
    handle_table_appearance_sliders, poll_table_felt_picker, sync_table_appearance,
    update_settings_tab_hover,
};
use crate::app::runtime::ClientUpdateSet;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::runtime::{AiModels, initialize_ai_models, poll_ai_models, sync_lobby_ai};
use crate::app::shell::{ModalAnimationSet, UiActionSet, animate_cozy_modals};
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use leocard_ui::TextInputSet;

pub(crate) struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.init_resource::<AiModels>()
            .add_systems(
                PostUpdate,
                sync_ai_settings_inputs.in_set(TextInputSet::Publish),
            )
            .add_systems(Startup, initialize_ai_models)
            .add_systems(
                Update,
                (poll_ai_models, sync_lobby_ai)
                    .chain()
                    .in_set(ClientUpdateSet::Sync),
            );
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
                update_settings_tab_hover
                    .after(animate_cozy_modals)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
