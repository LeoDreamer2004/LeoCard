use super::{
    CardDragSelection, GameSummaryAnimation, StartGameSeatTransition, SummaryPlayback,
    TableBackgroundMaterial, TurnBorderAnimationState, TurnBorderMaterial, animate_button_arrows,
    animate_game_summary_visuals, animate_signed_summary_scores,
    animate_start_game_seat_transition, animate_summary_scores, animate_turn_border_traces,
    update_button_highlights, update_summary_animation,
};
use super::{animate_button_presses, play_button_click_sounds, update_button_tints};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;
use bevy::ui_render::UiMaterialPlugin;

pub(crate) struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<TableBackgroundMaterial>::default())
            .add_plugins(UiMaterialPlugin::<TurnBorderMaterial>::default())
            .init_resource::<SummaryPlayback>()
            .insert_resource(GameSummaryAnimation::default())
            .insert_resource(StartGameSeatTransition::default())
            .insert_resource(TurnBorderAnimationState::default())
            .insert_resource(CardDragSelection::default())
            .add_systems(
                Update,
                (update_button_highlights, animate_button_arrows).in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    update_button_tints,
                    play_button_click_sounds,
                    animate_button_presses,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                Update,
                update_summary_animation.in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (
                    animate_game_summary_visuals,
                    animate_summary_scores,
                    animate_signed_summary_scores,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    animate_start_game_seat_transition,
                    animate_turn_border_traces,
                )
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
