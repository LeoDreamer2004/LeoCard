use super::{
    AchievementsPagePlugin, ChatPlugin, ConnectionPlugin, DeveloperToolsPlugin, InputPlugin,
    LobbyPlugin, NavigationPlugin, OverlaysPlugin, ProfilePlugin, SettingsPlugin, ShellUiPlugin,
    ShopPlugin, SocialPlugin, UiActionPlugin, UiActionSet, UiState, UpdatePlugin,
    close_interaction_menu_on_outside_click, rebuild_ui,
};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct ShellPlugin;

impl Plugin for ShellPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiState {
            dirty: true,
            ..default()
        })
        .add_plugins((
            UiActionPlugin,
            InputPlugin,
            OverlaysPlugin,
            AchievementsPagePlugin,
            ShopPlugin,
            ShellUiPlugin,
            ChatPlugin,
            ConnectionPlugin,
            DeveloperToolsPlugin,
            LobbyPlugin,
            NavigationPlugin,
            ProfilePlugin,
            SettingsPlugin,
            SocialPlugin,
            UpdatePlugin,
        ))
        .configure_sets(
            Update,
            UiActionSet.before(close_interaction_menu_on_outside_click),
        )
        .add_systems(Update, rebuild_ui.in_set(ClientUpdateSet::Rebuild));
    }
}
