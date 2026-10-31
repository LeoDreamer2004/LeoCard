use super::{
    CardVoices, GameSummaryAnimation, PlayEffectState, StartGameSeatTransition,
    SummaryAnimationSet, SummaryPlayback, TableBackgroundMaterial, TableStatusAnimation,
    TurnBorderAnimationState, TurnBorderMaterial, advance_play_effect, animate_bomb_play_effect,
    animate_button_arrows, animate_game_summary_visuals, animate_heaven_bomb_play_effect,
    animate_no_legal_response_hint, animate_playing_card_flips, animate_sequence_play_effect,
    animate_start_game_seat_transition, animate_summary_scores, animate_table_piece_deals,
    animate_table_statuses, animate_turn_border_traces, play_card_voices, play_ui_click_sounds,
    resolve_table_piece_origins, start_game_intro_finished, sync_card_voices,
    update_summary_animation,
};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy::ui_render::UiMaterialPlugin;
use leocard_ui::{
    CardDragSelection, PointerInputPlugin, ScrollbarDrag, TextInputPlugin, UiSceneProperties,
    UiSceneRegistry, animate_bubbles, animate_button_presses, animate_drawers,
    animate_progress_bars, drag_scrollbar, image_node_eq, position_anchored_popups,
    update_button_highlights, update_button_tints, update_scrollbar,
};

pub(crate) struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        let mut properties = UiSceneProperties::default();
        properties.preserve::<Node>();
        properties.preserve::<UiTransform>();
        properties.preserve::<Text>();
        properties.preserve::<TextFont>();
        properties.preserve_with::<TextLayout>(|left, right| {
            left.justify == right.justify && left.linebreak == right.linebreak
        });
        properties.preserve::<TextColor>();
        properties.preserve::<TableStatusAnimation>();
        properties.preserve::<BackgroundColor>();
        properties.preserve::<BorderColor>();
        properties.preserve::<BackgroundGradient>();
        properties.preserve::<Visibility>();
        properties.preserve::<ZIndex>();
        properties.preserve_with::<ScrollPosition>(|left, right| left.0 == right.0);
        properties.preserve_with::<ImageNode>(image_node_eq);
        app.add_systems(
            Update,
            (animate_bubbles, animate_drawers).in_set(ClientUpdateSet::Animate),
        )
        .add_plugins(PointerInputPlugin)
        .init_resource::<UiSceneRegistry>()
        .insert_resource(properties)
        .add_plugins(TextInputPlugin)
        .add_plugins(UiMaterialPlugin::<TableBackgroundMaterial>::default())
        .add_plugins(UiMaterialPlugin::<TurnBorderMaterial>::default())
        .init_resource::<SummaryPlayback>()
        .init_resource::<CardVoices>()
        .init_resource::<PlayEffectState>()
        .insert_resource(GameSummaryAnimation::default())
        .insert_resource(StartGameSeatTransition::default())
        .insert_resource(TurnBorderAnimationState::default())
        .insert_resource(CardDragSelection::default())
        .init_resource::<ScrollbarDrag>()
        .add_systems(Update, sync_card_voices.in_set(ClientUpdateSet::Sync))
        .add_systems(
            Update,
            play_card_voices
                .run_if(start_game_intro_finished)
                .in_set(ClientUpdateSet::Animate),
        )
        .add_systems(
            Update,
            (
                advance_play_effect,
                animate_sequence_play_effect,
                animate_bomb_play_effect,
                animate_heaven_bomb_play_effect,
            )
                .chain()
                .run_if(start_game_intro_finished)
                .in_set(ClientUpdateSet::Animate),
        )
        .add_systems(
            Update,
            (
                animate_table_piece_deals,
                animate_playing_card_flips,
                animate_no_legal_response_hint,
                animate_table_statuses,
            )
                .chain()
                .run_if(start_game_intro_finished)
                .in_set(ClientUpdateSet::Animate),
        )
        .add_systems(
            PostUpdate,
            resolve_table_piece_origins
                .after(UiSystems::Layout)
                .before(UiSystems::PostLayout),
        )
        .add_systems(Update, drag_scrollbar.in_set(ClientUpdateSet::Input))
        .add_systems(
            Update,
            animate_progress_bars.in_set(ClientUpdateSet::Animate),
        )
        .add_systems(PostUpdate, update_scrollbar.after(UiSystems::Layout))
        .add_systems(
            PostUpdate,
            position_anchored_popups
                .after(UiSystems::Layout)
                .before(UiSystems::PostLayout),
        )
        .add_systems(
            Update,
            (update_button_highlights, animate_button_arrows).in_set(ClientUpdateSet::Animate),
        )
        .add_systems(
            Update,
            (
                update_button_tints,
                play_ui_click_sounds,
                animate_button_presses,
            )
                .chain()
                .in_set(ClientUpdateSet::Input),
        )
        .add_systems(
            Update,
            update_summary_animation
                .in_set(SummaryAnimationSet)
                .in_set(ClientUpdateSet::Sync),
        )
        .add_systems(
            Update,
            (animate_game_summary_visuals, animate_summary_scores)
                .chain()
                .in_set(SummaryAnimationSet)
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
