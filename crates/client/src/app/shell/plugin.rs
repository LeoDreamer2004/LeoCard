use super::{
    ChatPlugin, ConnectionPlugin, DeveloperToolsPlugin, LobbyPlugin, NavigationPlugin,
    PlayErrorToast, SettingsPlugin, SocialPlugin, UiActionPlugin, UiActionSet, UiState, UiZoom,
    UpdatePlugin, animate_button_presses, close_interaction_menu_on_outside_click,
    handle_text_input, play_button_click_sounds, rebuild_ui, sync_ime_enabled, update_button_tints,
    update_ui_scale,
};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayErrorToast::default())
            .insert_resource(UiState {
                dirty: true,
                ..default()
            })
            .insert_resource(UiZoom::default())
            .add_plugins((
                UiActionPlugin,
                ChatPlugin,
                ConnectionPlugin,
                DeveloperToolsPlugin,
                LobbyPlugin,
                NavigationPlugin,
                SettingsPlugin,
                SocialPlugin,
                UpdatePlugin,
            ))
            .configure_sets(
                Update,
                UiActionSet.before(close_interaction_menu_on_outside_click),
            )
            .add_systems(
                Update,
                (
                    sync_ime_enabled,
                    update_ui_scale,
                    handle_text_input,
                    update_button_tints,
                    play_button_click_sounds,
                    animate_button_presses,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(Update, rebuild_ui.in_set(ClientUpdateSet::Rebuild));
    }
}
