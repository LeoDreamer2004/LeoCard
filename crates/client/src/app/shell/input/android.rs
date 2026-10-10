use crate::app::shell::{
    ConfirmationDialog, ConfirmationUiAction, NavigationUiAction, PressedUiAction, UiAction,
    UiState,
};
use bevy::prelude::*;
use leocard_client::platform::{background_app, take_back_request};

pub(super) fn handle_back(
    ui: Res<UiState>,
    confirmation: Res<ConfirmationDialog>,
    mut actions: MessageWriter<PressedUiAction>,
) {
    if !take_back_request() {
        return;
    }
    let action = if confirmation.is_open() {
        Some(UiAction::Confirmation(ConfirmationUiAction::Cancel))
    } else if ui.settings.open {
        Some(UiAction::Navigation(NavigationUiAction::ToggleSettings))
    } else if ui.profile.open {
        Some(UiAction::Navigation(NavigationUiAction::ToggleProfile))
    } else if ui.shop.open {
        Some(UiAction::Navigation(NavigationUiAction::ToggleShop))
    } else if ui.achievements.open {
        Some(UiAction::Navigation(NavigationUiAction::ToggleAchievements))
    } else {
        None
    };
    if let Some(action) = action {
        actions.write(PressedUiAction(action));
    } else if let Err(error) = background_app() {
        warn!("{error}");
    }
}
