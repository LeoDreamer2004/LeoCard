use super::super::{TexasRaiseAdjustButton, TexasUiAction, TexasUiState};
use super::slider::raise_surface;
use super::{TexasRaiseFill, raise_fraction};
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui::BackgroundGradient;

/// Update the few mutable controls without composing the rest of the table.
pub(in super::super) fn sync_texas_raise_controls(
    state: Res<TexasUiState>,
    mut fills: Query<(&mut TexasRaiseFill, &mut Node, &mut BackgroundGradient)>,
    mut adjusters: Query<(&TexasRaiseAdjustButton, &mut UiAction)>,
    mut previous: Local<Option<u32>>,
) {
    let target = state.raise_to;
    if *previous == Some(target) {
        return;
    }
    *previous = Some(target);
    for (mut fill, mut node, mut surface) in &mut fills {
        if fill.target == target {
            continue;
        }
        let fraction = raise_fraction(fill.sizing, target);
        node.width = percent(3.5 + 96.5 * fraction);
        *surface = raise_surface(fraction);
        fill.target = target;
    }
    for (button, mut action) in &mut adjusters {
        let next = if button.direction < 0 {
            target.saturating_sub(button.step).max(button.minimum)
        } else {
            target.saturating_add(button.step).min(button.maximum)
        };
        *action = UiAction::Texas(TexasUiAction::SetRaiseTo(next));
    }
}
