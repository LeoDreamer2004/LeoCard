use super::actions::dispatch_actions;
use super::counter::sync_counter;
use super::feedback::{advance_previous_trick, queue_deal, sync_play_feedback};
use super::input::{animate_hand_cards, handle_card_drag_selection, sync_card_drag_preview};
use super::state::GuandanUiState;
use super::voices::sync_guandan_voices;
use crate::app::presentation::{start_game_intro_finished, sync_card_voices};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;
use leocard_ui::drag_draggable_window;

pub(crate) struct GuandanPlugin;

impl Plugin for GuandanPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuandanUiState>()
            .add_systems(Update, dispatch_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                (sync_counter, sync_play_feedback).in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                sync_guandan_voices
                    .after(sync_card_voices)
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                queue_deal
                    .after(sync_play_feedback)
                    .run_if(start_game_intro_finished)
                    .in_set(ClientUpdateSet::Sync),
            )
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
                (
                    sync_card_drag_preview,
                    animate_hand_cards,
                    advance_previous_trick,
                )
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
