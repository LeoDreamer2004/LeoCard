use super::back::back_action;
use crate::app::shell::{ConfirmationDialog, PressedUiAction, UiState};
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
    if let Some(action) = back_action(&ui, &confirmation) {
        actions.write(PressedUiAction(action));
    } else if let Err(error) = background_app() {
        warn!("{error}");
    }
}
