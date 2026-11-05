use crate::app::shell::{
    ConfirmationDialog, ConfirmationUiAction, ConnectionUiAction, MusicMotion, NavigationUiAction,
    PageMotion, PressedUiAction, UiAction, UiState,
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
    } else if ui.home.identity_open {
        Some(UiAction::Connection(ConnectionUiAction::ToggleIdentity))
    } else if ui.home.join_open {
        Some(UiAction::Connection(ConnectionUiAction::ToggleJoin))
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
    music: Res<MusicMotion>,
) {
    if !keyboard.just_pressed(KeyCode::Escape)
        || page_motion.active()
        || !(confirmation.is_open()
            || ui.shop.open
            || ui.achievements.open
            || ui.lan.open
            || ui.home.identity_open
            || ui.home.join_open
            || music.target_open)
    {
        return;
    }
    let action = if music.target_open && !confirmation.is_open() {
        Some(UiAction::Navigation(NavigationUiAction::ToggleMusic))
    } else {
        back_action(&ui, &confirmation)
    };
    if let Some(action) = action {
        if matches!(&action, UiAction::Navigation(_)) {
            ui.dirty = true;
        }
        actions.write(PressedUiAction(action));
    }
}
