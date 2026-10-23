use crate::app::shell::{
    ConfirmationDialog, ConfirmationUiAction, ConnectionUiAction, NavigationUiAction, PageMotion,
    PressedUiAction, UiAction, UiState,
};
use bevy::prelude::*;

pub(super) fn back_action(ui: &UiState, confirmation: &ConfirmationDialog) -> Option<UiAction> {
    if confirmation.is_open() {
        Some(UiAction::Confirmation(ConfirmationUiAction::Cancel))
    } else if ui.lan.open {
        Some(UiAction::Connection(ConnectionUiAction::CloseLan))
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
    }
}

pub(super) fn handle_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui: ResMut<UiState>,
    confirmation: Res<ConfirmationDialog>,
    page_motion: Res<PageMotion>,
    mut actions: MessageWriter<PressedUiAction>,
) {
    if !keyboard.just_pressed(KeyCode::Escape)
        || page_motion.active()
        || !(confirmation.is_open() || ui.shop.open || ui.achievements.open || ui.lan.open)
    {
        return;
    }
    if let Some(action) = back_action(&ui, &confirmation) {
        if matches!(&action, UiAction::Navigation(_)) {
            ui.dirty = true;
        }
        actions.write(PressedUiAction(action));
    }
}
