use super::{
    GameSummaryAnimation, StartGameSeatTransition, TableBackgroundMaterial,
    TurnBorderAnimationState, TurnBorderMaterial, animate_game_summary_visuals,
    animate_signed_summary_scores, animate_start_game_seat_transition, animate_summary_scores,
    animate_turn_border_traces, update_summary_animation,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::CardDragSelection;
use bevy::prelude::*;
use bevy::ui_render::UiMaterialPlugin;

pub(crate) struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<TableBackgroundMaterial>::default())
            .add_plugins(UiMaterialPlugin::<TurnBorderMaterial>::default())
            .insert_resource(GameSummaryAnimation::default())
            .insert_resource(StartGameSeatTransition::default())
            .insert_resource(TurnBorderAnimationState::default())
            .insert_resource(CardDragSelection::default())
            .add_systems(
                Update,
                (
                    update_summary_animation,
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
