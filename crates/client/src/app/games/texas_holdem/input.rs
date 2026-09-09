//! 德州下注按钮的持续按压输入。

use super::{TexasHoldemUiState, TexasRaiseAdjustButton, TexasRaiseHoldState};
use crate::app::shell::UiState;
use bevy::prelude::*;

pub(crate) fn handle_texas_raise_button_hold(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    changed: Query<(&Interaction, &TexasRaiseAdjustButton), (Changed<Interaction>, With<Button>)>,
    mut hold: ResMut<TexasRaiseHoldState>,
    mut game_ui: ResMut<TexasHoldemUiState>,
    mut ui: ResMut<UiState>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Some((_, button)) = changed
            .iter()
            .find(|(interaction, _)| **interaction == Interaction::Pressed)
        {
            *hold = TexasRaiseHoldState {
                direction: button.direction,
                step: button.step,
                minimum: button.minimum,
                maximum: button.maximum,
                elapsed: 0.0,
                next_repeat: 0.42,
            };
        }
        return;
    }
    if mouse.just_released(MouseButton::Left) || !mouse.pressed(MouseButton::Left) {
        *hold = TexasRaiseHoldState::default();
        return;
    }
    if hold.direction == 0 {
        return;
    }

    hold.elapsed += time.delta_secs();
    while hold.elapsed >= hold.next_repeat {
        let current = game_ui.raise_to;
        let next = texas_raise_repeat_value(
            current,
            hold.direction,
            hold.step,
            hold.minimum,
            hold.maximum,
        );
        if next == current {
            hold.direction = 0;
            break;
        }
        game_ui.raise_to = next;
        ui.dirty = true;
        hold.next_repeat += if hold.elapsed >= 1.35 { 0.065 } else { 0.11 };
    }
}

pub(crate) fn texas_raise_repeat_value(
    current: u32,
    direction: i8,
    step: u32,
    minimum: u32,
    maximum: u32,
) -> u32 {
    if direction < 0 {
        current.saturating_sub(step).max(minimum)
    } else if direction > 0 {
        current.saturating_add(step).min(maximum)
    } else {
        current.clamp(minimum, maximum)
    }
}
