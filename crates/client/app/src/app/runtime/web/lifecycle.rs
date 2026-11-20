use super::super::{ClientResource, ConnectionDraft};
use crate::app::shell::{
    ConnectionUiAction, PressedUiAction, UiAction, UiState, can_edit_identity,
};
use crate::network::pending_web_room;
use bevy::prelude::*;
use bevy::window::{FileDragAndDrop, PrimaryWindow};
use leocard_client::ClientPhaseRef;

pub(super) fn mark_started() {
    if let Some(canvas) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("leocard"))
    {
        let _ = canvas.set_attribute("data-started", "true");
    }
}

pub(super) fn restore_room(
    mut connection: ResMut<ConnectionDraft>,
    mut ui: ResMut<UiState>,
    mut actions: MessageWriter<PressedUiAction>,
) {
    if let Some(address) = pending_web_room() {
        connection.join_address = address;
        ui.rooms.set_open(true);
        actions.write(PressedUiAction(UiAction::Connection(
            ConnectionUiAction::JoinRoom,
        )));
    }
}

pub(super) fn sync_active_room(client: Option<Res<ClientResource>>) {
    let active = client.is_some_and(|client| {
        matches!(
            client.0.model().phase(),
            ClientPhaseRef::Lobby(_) | ClientPhaseRef::Playing(_)
        )
    });
    leocard_platform::set_room_active(active);
}

pub(super) fn sync_image_drop(
    ui: Res<UiState>,
    client: Option<Res<ClientResource>>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut files: MessageWriter<FileDragAndDrop>,
) {
    leocard_platform::enable_image_drop(
        ui.home.identity_open && can_edit_identity(client.as_deref()),
    );
    if let Ok(window) = windows.single() {
        for path_buf in leocard_platform::take_dropped_images() {
            files.write(FileDragAndDrop::DroppedFile { window, path_buf });
        }
    }
}
