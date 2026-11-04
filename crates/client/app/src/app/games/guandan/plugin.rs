use super::actions::dispatch_actions;
use super::counter::sync_counter;
use super::input::{animate_hand_cards, handle_card_drag_selection, sync_card_drag_preview};
use super::state::GuandanUiState;
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;
use leocard_ui::drag_draggable_window;

pub(crate) struct GuandanPlugin;

impl Plugin for GuandanPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuandanUiState>()
            .add_systems(Update, dispatch_actions.in_set(UiActionSet))
            .add_systems(Update, sync_counter.in_set(ClientUpdateSet::Sync))
            .add_systems(
                Update,
                (
                    drag_draggable_window::<GuandanUiState>,
                    handle_card_drag_selection,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                (sync_card_drag_preview, animate_hand_cards).in_set(ClientUpdateSet::Animate),
            );
    }
}
