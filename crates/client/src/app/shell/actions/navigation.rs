//! 个人资料、设置、更新与桌面外观动作。

use super::super::{
    UiState, UpdateManager, UpdateState, open_github_repository, start_table_felt_picker,
};
use super::{
    AppearanceUiResources, NavigationUiAction, PressedUiAction, UiActionHandler,
    dispatch_domain_actions,
};
use crate::app::runtime::{AppearancePreferences, save_appearance_preferences};
use bevy::ecs::system::SystemParam;
use bevy::log::warn;
use bevy::prelude::*;

#[derive(SystemParam)]
pub(crate) struct NavigationActionContext<'w> {
    appearance: ResMut<'w, AppearancePreferences>,
    ui: ResMut<'w, UiState>,
    local: AppearanceUiResources<'w>,
    updater: ResMut<'w, UpdateManager>,
    app_exit: MessageWriter<'w, AppExit>,
}

pub(crate) fn dispatch_navigation_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: NavigationActionContext,
) {
    dispatch_domain_actions::<NavigationUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<NavigationActionContext<'_>> for NavigationUiAction {
    fn handle(&self, context: &mut NavigationActionContext<'_>) {
        let appearance = &mut *context.appearance;
        let ui = &mut *context.ui;
        let local = &mut context.local;
        let updater = &mut *context.updater;
        match self {
            NavigationUiAction::ToggleProfile => {
                ui.navigation.profile_open = !ui.navigation.profile_open;
                ui.navigation.player_profile = None;
                if ui.navigation.profile_open {
                    ui.navigation.settings_open = false;
                    ui.navigation.host_game_picker_open = false;
                }
            }
            NavigationUiAction::OpenPlayerProfile(player_profile) => {
                ui.navigation.profile_open = true;
                ui.navigation.player_profile = Some((**player_profile).clone());
                ui.social.interaction_menu_open = None;
                ui.navigation.settings_open = false;
                ui.navigation.host_game_picker_open = false;
            }
            NavigationUiAction::SelectProfileGameTab(tab) => ui.navigation.profile_game_tab = *tab,
            NavigationUiAction::ToggleSettings => {
                ui.navigation.settings_open = !ui.navigation.settings_open;
                if ui.navigation.settings_open {
                    ui.navigation.profile_open = false;
                    ui.navigation.player_profile = None;
                    ui.navigation.host_game_picker_open = false;
                }
            }
            NavigationUiAction::StartUpdate => updater.begin_or_show(),
            NavigationUiAction::OpenGitHubRepository => {
                if let Err(error) = open_github_repository() {
                    warn!("{error}");
                }
            }
            NavigationUiAction::HideUpdateDialog => updater.dialog_open = false,
            NavigationUiAction::RestartToUpdate => {
                restart_to_update(updater, &mut context.app_exit)
            }
            NavigationUiAction::ChooseTableFelt => {
                if local.table_felt_picker.pending.is_none() {
                    match start_table_felt_picker() {
                        Ok(receiver) => {
                            local.table_felt_picker.pending = Some(receiver);
                            local.table_appearance.error = None;
                        }
                        Err(error) => local.table_appearance.error = Some(error),
                    }
                }
            }
            NavigationUiAction::UseDefaultTableFelt => {
                appearance.table_felt_path = None;
                appearance.table_brightness = 1.0;
                local.table_appearance.error = save_appearance_preferences(appearance).err();
            }
        }
    }
}

fn restart_to_update(updater: &mut UpdateManager, app_exit: &mut MessageWriter<AppExit>) {
    let UpdateState::Ready { staged, .. } = &updater.state else {
        return;
    };
    match crate::updater::launch_installer(staged) {
        Ok(()) => {
            app_exit.write(AppExit::Success);
        }
        Err(error) => {
            updater.state = UpdateState::Failed(error);
            updater.dialog_open = true;
        }
    }
}
