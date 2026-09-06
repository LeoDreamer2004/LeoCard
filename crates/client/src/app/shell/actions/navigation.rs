//! 个人资料、设置、更新与桌面外观动作。

use super::*;

pub fn handle_navigation_button(
    action: &UiAction,
    form: &mut ConnectionForm,
    ui: &mut UiState,
    local: &mut LocalUiResources<'_>,
    updater: &mut UpdateManager,
    app_exit: &mut MessageWriter<AppExit>,
) -> bool {
    match action {
        UiAction::ToggleProfile => {
            ui.navigation.profile_open = !ui.navigation.profile_open;
            ui.navigation.player_profile = None;
            if ui.navigation.profile_open {
                ui.navigation.settings_open = false;
                ui.navigation.host_game_picker_open = false;
            }
        }
        UiAction::OpenPlayerProfile(player_profile) => {
            ui.navigation.profile_open = true;
            ui.navigation.player_profile = Some((**player_profile).clone());
            ui.social.interaction_menu_open = None;
            ui.navigation.settings_open = false;
            ui.navigation.host_game_picker_open = false;
        }
        UiAction::SelectProfileGameTab(tab) => ui.navigation.profile_game_tab = *tab,
        UiAction::ToggleSettings => {
            ui.navigation.settings_open = !ui.navigation.settings_open;
            if ui.navigation.settings_open {
                ui.navigation.profile_open = false;
                ui.navigation.player_profile = None;
                ui.navigation.host_game_picker_open = false;
            }
        }
        UiAction::StartUpdate => updater.begin_or_show(),
        UiAction::OpenGitHubRepository => {
            if let Err(error) = open_github_repository() {
                warn!("{error}");
            }
        }
        UiAction::HideUpdateDialog => updater.dialog_open = false,
        UiAction::RestartToUpdate => restart_to_update(updater, app_exit),
        UiAction::ChooseTableFelt => {
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
        UiAction::UseDefaultTableFelt => {
            restore_default_table_appearance(form);
            local.table_appearance.error = save_preferences(form).err();
        }
        _ => return false,
    }
    true
}

fn restore_default_table_appearance(form: &mut ConnectionForm) {
    form.table_felt_path = None;
    form.table_brightness = 1.0;
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
