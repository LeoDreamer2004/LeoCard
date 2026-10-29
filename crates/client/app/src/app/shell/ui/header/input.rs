use super::state::{GameHeaderAvatar, GameHeaderMotion};
use crate::app::shell::{NavigationUiAction, PressedUiAction, UiAction, UiState};
use bevy::prelude::*;
use leocard_ui::UiPress;

pub(super) fn handle_header_taps(
    mouse: Res<ButtonInput<MouseButton>>,
    mut presses: MessageReader<UiPress>,
    avatars: Query<(), With<GameHeaderAvatar>>,
    mut headers: Query<&mut GameHeaderMotion>,
    mut actions: MessageWriter<PressedUiAction>,
    mut ui: ResMut<UiState>,
) {
    let mut avatar_pressed = false;
    for press in presses.read() {
        avatar_pressed |= avatars.contains(press.0);
    }
    for mut motion in &mut headers {
        if avatar_pressed {
            if motion.expanded {
                motion.expanded = false;
                ui.dirty = true;
                actions.write(PressedUiAction(UiAction::Navigation(
                    NavigationUiAction::ToggleProfile,
                )));
            } else {
                motion.expanded = true;
            }
        } else if mouse.just_pressed(MouseButton::Left) && motion.expanded {
            motion.expanded = false;
        }
    }
}
