use super::back::dispatch_back;
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    LobbyUiAction, ModalInputScope, PageMotion, PressedUiAction, UiAction, UiState,
};
use bevy::prelude::*;
use leocard_platform::{allow_browser_back, take_browser_back};

pub(super) fn handle_back(
    mut ui: ResMut<UiState>,
    scope: ModalInputScope,
    page_motion: Res<PageMotion>,
    client: Option<Res<ClientResource>>,
    mut actions: MessageWriter<PressedUiAction>,
) {
    if !take_browser_back() || page_motion.active() {
        return;
    }
    if !dispatch_back(&mut ui, &scope, &mut actions) {
        if client.is_some() {
            actions.write(PressedUiAction(UiAction::Lobby(
                LobbyUiAction::RequestLeaveRoom,
            )));
        } else {
            allow_browser_back();
        }
    }
}
