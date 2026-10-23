use super::UpdateManager;
use crate::app::shell::{CozyModalKind, ModalAnimations, UiState, advance_modal};
use bevy::prelude::*;

pub(super) fn advance_update_modal(
    time: Res<Time>,
    mut state: ResMut<UpdateManager>,
    mut animations: ResMut<ModalAnimations>,
    mut ui: ResMut<UiState>,
) {
    let open = state.dialog_open;
    let previous = state.dialog_progress;
    advance_modal(&mut state.dialog_progress, open, time.delta_secs());
    animations.set(CozyModalKind::UpdateDialog, state.dialog_progress);
    if !open && previous > 0.0 && state.dialog_progress == 0.0 {
        ui.dirty = true;
    }
}
