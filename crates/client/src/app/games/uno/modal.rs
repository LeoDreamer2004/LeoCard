use super::UnoUiState;
use crate::app::shell::{CozyModalKind, ModalAnimations, UiState, advance_modal};
use bevy::prelude::*;

pub(super) fn advance_uno_modal(
    time: Res<Time>,
    mut state: ResMut<UnoUiState>,
    mut animations: ResMut<ModalAnimations>,
    mut ui: ResMut<UiState>,
) {
    let open = state.expansion_settings_open;
    let previous = state.expansion_settings_progress;
    advance_modal(
        &mut state.expansion_settings_progress,
        open,
        time.delta_secs(),
    );
    animations.set(
        CozyModalKind::UnoExpansionSettings,
        state.expansion_settings_progress,
    );
    if !open && previous > 0.0 && state.expansion_settings_progress == 0.0 {
        ui.dirty = true;
    }
}
