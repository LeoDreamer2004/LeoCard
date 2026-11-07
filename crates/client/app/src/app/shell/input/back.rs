use crate::app::shell::{
    ModalInputScope, NavigationUiAction, PageMotion, PressedUiAction, UiAction, UiState,
};
use bevy::prelude::*;

pub(super) fn dispatch_back(
    ui: &mut UiState,
    scope: &ModalInputScope,
    actions: &mut MessageWriter<PressedUiAction>,
) -> bool {
    let action = if let Some((_, dismiss)) = scope.top() {
        let Some(dismiss) = dismiss else {
            return true;
        };
        dismiss.0.clone()
    } else if ui.shop.open {
        UiAction::Navigation(NavigationUiAction::ToggleShop)
    } else if ui.achievements.open {
        UiAction::Navigation(NavigationUiAction::ToggleAchievements)
    } else {
        return false;
    };
    ui.dirty |= action.rebuilds_ui();
    actions.write(PressedUiAction(action));
    true
}

pub(super) fn handle_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui: ResMut<UiState>,
    scope: ModalInputScope,
    page_motion: Res<PageMotion>,
    mut actions: MessageWriter<PressedUiAction>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && !page_motion.active() {
        dispatch_back(&mut ui, &scope, &mut actions);
    }
}
