use super::back::dispatch_back;
use crate::app::shell::{ModalInputScope, PageMotion, PressedUiAction, UiState};
use bevy::prelude::*;
use leocard_client::platform::{background_app, take_back_request};

pub(super) fn handle_back(
    mut ui: ResMut<UiState>,
    scope: ModalInputScope,
    page_motion: Res<PageMotion>,
    mut actions: MessageWriter<PressedUiAction>,
) {
    if !take_back_request() || page_motion.active() {
        return;
    }
    if !dispatch_back(&mut ui, &scope, &mut actions)
        && let Err(error) = background_app()
    {
        warn!("{error}");
    }
}
