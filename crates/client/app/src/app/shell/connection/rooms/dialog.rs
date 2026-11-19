use crate::app::shell::{CozyModalKind, ModalAnimations, UiState, advance_modal};
use bevy::prelude::*;

pub(in super::super) fn advance_room_dialog(
    time: Res<Time>,
    mut ui: ResMut<UiState>,
    mut animations: ResMut<ModalAnimations>,
) {
    let room = &mut ui.rooms;
    let previous = room.dialog_progress;
    if !room.open {
        room.dialog_open = false;
        room.dialog_progress = 0.0;
    } else {
        advance_modal(
            &mut room.dialog_progress,
            room.dialog_open,
            time.delta_secs(),
        );
    }
    animations.set(CozyModalKind::JoinRoomEditor, room.dialog_progress);
    if !room.dialog_open && room.dialog_progress == 0.0 {
        room.dialog = None;
        if previous > 0.0 {
            ui.dirty = true;
        }
    }
}
